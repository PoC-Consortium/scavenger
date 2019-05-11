use crate::com::api::{FetchError, MiningInfoResponse};
use crate::com::client::{Client, ProxyDetails, SubmissionParameters};
use crate::future::prio_retry::PrioRetry;
use futures::future::Future;
use futures::stream::Stream;
use futures::sync::mpsc;
use std::collections::HashMap;
use std::time::Duration;
use std::u64;
use tokio;
use tokio::runtime::TaskExecutor;
use url::Url;

#[derive(Clone)]
pub struct RequestHandler {
    client: Client,
    tx_submit_data: mpsc::UnboundedSender<SubmissionParameters>,
}

impl RequestHandler {
    pub fn new(
        base_uri: Url,
        secret_phrases: HashMap<u64, String>,
        timeout: u64,
        total_size_gb: usize,
        send_proxy_details: bool,
        additional_headers: HashMap<String, String>,
        executor: TaskExecutor,
    ) -> RequestHandler {
        // TODO
        let proxy_details = if send_proxy_details {
            ProxyDetails::Enabled
        } else {
            ProxyDetails::Disabled
        };

        let client = Client::new(
            base_uri,
            secret_phrases,
            timeout,
            total_size_gb,
            proxy_details,
            additional_headers,
        );

        let (tx_submit_data, rx_submit_nonce_data) = mpsc::unbounded();
        RequestHandler::handle_submissions(
            client.clone(),
            rx_submit_nonce_data,
            tx_submit_data.clone(),
            executor,
        );

        RequestHandler {
            client,
            tx_submit_data,
        }
    }

    fn handle_submissions(
        client: Client,
        rx: mpsc::UnboundedReceiver<SubmissionParameters>,
        tx_submit_data: mpsc::UnboundedSender<SubmissionParameters>,
        executor: TaskExecutor,
    ) {
        let stream = PrioRetry::new(rx, Duration::from_secs(3))
            .and_then(move |submission_params| {
                let tx_submit_data = tx_submit_data.clone();
                client
                    .clone()
                    .submit_nonce(&submission_params)
                    .then(move |res| {
                        match res {
                            Ok(res) => {
                                if submission_params.deadline != res.deadline {
                                    log_deadline_mismatch(
                                        submission_params.height,
                                        submission_params.account_id,
                                        submission_params.nonce,
                                        submission_params.deadline,
                                        res.deadline,
                                    );
                                } else {
                                    log_submission_accepted(
                                        submission_params.account_id,
                                        submission_params.nonce,
                                        submission_params.deadline,
                                    );
                                }
                            }
                            Err(FetchError::Pool(e)) => {
                                log_submission_not_accepted(
                                    submission_params.height,
                                    submission_params.account_id,
                                    submission_params.nonce,
                                    submission_params.deadline,
                                    e.code,
                                    &e.message,
                                );
                                let res = tx_submit_data.unbounded_send(submission_params);
                                if let Err(e) = res {
                                    error!("can't send submission params: {}", e);
                                }
                            }
                            Err(_) => {
                                log_submission_failed(
                                    0,
                                    submission_params.account_id,
                                    submission_params.nonce,
                                    submission_params.deadline,
                                );
                                let res = tx_submit_data.unbounded_send(submission_params);
                                if let Err(e) = res {
                                    error!("can't send submission params: {}", e);
                                }
                            }
                        };
                        Ok(())
                    })
            })
            .for_each(|_| Ok(()))
            .map_err(|e| error!("can't handle submission params: {:?}", e));
        executor.spawn(stream);
    }

    pub fn get_mining_info(&self) -> impl Future<Item = MiningInfoResponse, Error = FetchError> {
        self.client.get_mining_info()
    }

    pub fn submit_nonce(
        &self,
        account_id: u64,
        nonce: u64,
        height: u64,
        deadline_unadjusted: u64,
        deadline: u64,
        gen_sig: [u8; 32],
    ) {
        let res = self.tx_submit_data.unbounded_send(SubmissionParameters {
            account_id,
            nonce,
            height,
            deadline_unadjusted,
            deadline,
            gen_sig,
        });
        if let Err(e) = res {
            error!("can't send submission params: {}", e);
        }
    }
}

fn log_deadline_mismatch(
    height: u64,
    account_id: u64,
    nonce: u64,
    deadline: u64,
    deadline_pool: u64,
) {
    error!(
        "submit: deadlines mismatch, height={}, account={}, nonce={}, \
         deadline_miner={}, deadline_pool={}",
        height, account_id, nonce, deadline, deadline_pool
    );
}

fn log_submission_failed(retry: u8, account_id: u64, nonce: u64, deadline: u64) {
    if retry < 3 {
        warn!(
            "{: <80}",
            format!(
                "submission failed:, attempt={}, account={}, nonce={}, deadline={}",
                retry, account_id, nonce, deadline
            )
        );
    } else {
        error!(
            "{: <80}",
            format!(
                "submission retries exhausted: account={}, nonce={}, deadline={}",
                account_id, nonce, deadline
            )
        );
    }
}

fn log_submission_not_accepted(
    height: u64,
    account_id: u64,
    nonce: u64,
    deadline: u64,
    err_code: i32,
    msg: &str,
) {
    error!(
        "submission not accepted: height={}, account={}, nonce={}, \
         deadline={}\n\tcode: {}\n\tmessage: {}",
        height, account_id, nonce, deadline, err_code, msg,
    );
}

fn log_submission_accepted(account_id: u64, nonce: u64, deadline: u64) {
    info!(
        "deadline accepted: account={}, nonce={}, deadline={}",
        account_id, nonce, deadline
    );
}

use api;
use api_grpc;
use futures::{future, Future, Stream};
use grpcio::{ChannelBuilder, EnvBuilder};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::timer::Interval;

#[derive(Clone)]
pub struct Client {
    client: api_grpc::ApiClient,
    account_id_to_secret_phrase: HashMap<u64, String>,
}

impl Client {
    pub fn new(addr: &str, account_id_to_secret_phrase: HashMap<u64, String>) -> Client {
        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(addr);
        let client = api_grpc::ApiClient::new(ch);
        Client {
            client,
            account_id_to_secret_phrase,
        }
    }

    pub fn get_mining_info(&self) -> grpcio::ClientUnaryReceiver<api::GetMiningInfoResponse> {
        self.client
            .get_mining_info_async(&api::Void::new())
            .unwrap()
    }

    pub fn submit_nonce(&self, account_id: u64, nonce: u64, height: u64) -> impl Future {
        let client = self.client.clone();
        let secret_phrase = self
            .account_id_to_secret_phrase
            .get(&account_id)
            .unwrap_or(&"".to_owned())
            .to_string();

        let mut msg = api::SubmitNonceRequest::new();
        msg.set_account_id(account_id);
        msg.set_nonce(nonce);
        msg.set_height(height);
        msg.set_secret_phrase(secret_phrase);

        client
            .submit_nonce_async(&msg)
            .unwrap()
            .map_err(|_| retry_submit_nonce(client, msg))
    }
}

fn retry_submit_nonce(client: api_grpc::ApiClient, msg: api::SubmitNonceRequest) -> impl Future {
    let client = Arc::new(client);
    Interval::new_interval(Duration::from_secs(3))
        .take(3)
        .then(move |_| client.submit_nonce_async(&msg).unwrap())
        .then(|res| {
            let o: Result<Option<()>, ()> = match res {
                Err(_) => Ok(Some(())),
                Ok(_) => Ok(None),
            };
            o
        })
        .take_while(|res| future::ok(res.is_some()))
        .for_each(|_| Ok(()))
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::Future;

    #[derive(Clone)]
    struct ApiService;

    impl api_grpc::Api for ApiService {
        fn get_mining_info(
            &mut self,
            ctx: RpcContext,
            req: api::Void,
            sink: UnarySink<api::GetMiningInfoResponse>,
        ) {
            let mut resp = api::GetMiningInfoResponse::new();
            resp.set_height(1337);
            let f = sink
                .success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e));
            ctx.spawn(f)
        }

        fn submit_nonce(
            &mut self,
            ctx: RpcContext,
            req: api::SubmitNonceRequest,
            sink: UnarySink<api::SubmitNonceResponse>,
        ) {
            let mut resp = api::SubmitNonceResponse::new();

            assert_eq!(req.account_id, 1);
            assert_eq!(req.nonce, 2);
            assert_eq!(req.height, 3);
            assert_eq!(req.secret_phrase, "some secret phrase");

            resp.set_deadline(1337);
            let f = sink
                .success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e));
            ctx.spawn(f)
        }
    }

    #[test]
    fn test_client() {
        let env = Arc::new(Environment::new(1));
        let service = api_grpc::create_api(ApiService);
        let mut server = ServerBuilder::new(env)
            .register_service(service)
            .bind("127.0.0.1", 50_051)
            .build()
            .unwrap();
        server.start();

        let mut account_id_to_secret_phrase = HashMap::new();
        account_id_to_secret_phrase.insert(1, "some secret phrase".to_owned());

        let client = Client::new("127.0.0.1:50051", account_id_to_secret_phrase);

        tokio::run(client.get_mining_info().then(|res| {
            assert!(res.is_ok());
            let mining_info = res.unwrap();
            assert_eq!(mining_info.height, 1337);
            Ok(())
        }));

        tokio::run(client.submit_nonce(1, 2, 3).then(|res| {
            assert!(res.is_ok());
            Ok(())
        }));

        let _ = server.shutdown().wait();

        let f = retry_submit_nonce(client.client, api::SubmitNonceRequest::new()).then(|res| {
            assert!(res.is_ok());
            Ok(())
        });
        tokio::run(f);
    }
}

extern crate hyper;
extern crate hyper_rustls;
extern crate serde_json;
extern crate url;

use futures::future;
use hyper::client::HttpConnector;
use hyper::rt::{Future, Stream};
use hyper::{Client, Request};
use serde::de::{self, DeserializeOwned};
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::time::Duration;
use std::u64;
use tokio_core::reactor::{Handle, Timeout};
use url::form_urlencoded::byte_serialize;

#[derive(Clone)]
pub struct RequestHandler {
    account_id_to_secret_phrase: HashMap<u64, String>,
    base_uri: String,
    client: Client<hyper_rustls::HttpsConnector<HttpConnector>>,
    timeout: Duration,
    handle: Handle,
    ua: String,
}

pub enum FetchError {
    Http(hyper::Error),
    Pool(PoolError),
    Timeout(io::Error),
}

impl From<hyper::Error> for FetchError {
    fn from(err: hyper::Error) -> FetchError {
        FetchError::Http(err)
    }
}

impl From<PoolError> for FetchError {
    fn from(err: PoolError) -> FetchError {
        FetchError::Pool(err)
    }
}

impl From<io::Error> for FetchError {
    fn from(err: io::Error) -> FetchError {
        FetchError::Timeout(err)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MiningInfo {
    pub generation_signature: String,

    #[serde(deserialize_with = "from_str_or_int")]
    pub base_target: u64,

    #[serde(deserialize_with = "from_str_or_int")]
    pub height: u64,

    #[serde(default = "default_target_deadline")]
    pub target_deadline: u64,
}

fn default_target_deadline() -> u64 {
    u64::MAX
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitNonceResonse {
    pub deadline: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PoolErrorWrapper {
    error: PoolError,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolError {
    code: i32,
    message: String,
}

// MOTHERFUCKING pool
fn from_str_or_int<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct StringOrIntVisitor;

    impl<'de> de::Visitor<'de> for StringOrIntVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or int")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            v.parse::<u64>().map_err(de::Error::custom)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(v)
        }
    }

    deserializer.deserialize_any(StringOrIntVisitor)
}

impl RequestHandler {
    pub fn new(
        base_uri: String,
        mut secret_phrases: HashMap<u64, String>,
        timeout: u64,
        handle: Handle,
    ) -> RequestHandler {
        for secret_phrase in secret_phrases.values_mut() {
            *secret_phrase = byte_serialize(secret_phrase.as_bytes()).collect();
        }
        let https = hyper_rustls::HttpsConnector::new(4);
        let client: Client<_, hyper::Body> = Client::builder().build(https);

        RequestHandler {
            account_id_to_secret_phrase: secret_phrases,
            base_uri,
            client,
            timeout: Duration::from_millis(timeout),
            handle,
            ua: "scavenger/".to_owned() + crate_version!(),
        }
    }

    pub fn get_mining_info(&self) -> Box<Future<Item = MiningInfo, Error = FetchError>> {
        Box::new(self.do_req(self.get_req("/burst?requestType=getMiningInfo")))
    }

    pub fn submit_nonce(
        &self,
        handle: &Handle,
        account_id: u64,
        nonce: u64,
        height: u64,
        d: u64,
        retried: i32,
    ) {
        let empty = "".to_owned();
        let secret_phrase_encoded = self
            .account_id_to_secret_phrase
            .get(&account_id)
            .unwrap_or(&empty);

        let mut path = format!(
            "/burst?requestType=submitNonce&accountId={}&nonce={}&secretPhrase={}&blockheight={}",
            account_id, nonce, secret_phrase_encoded, height
        );
        // if pool mining also send the deadline (usefull for proxies)
        if secret_phrase_encoded == "" {
            path += &format!("&deadline={}", d);
        }

        let req = self.post_req(&path);

        let rh = self.clone();
        let inner_handle = handle.clone();
        handle.spawn(self.do_req(req).then(
            move |result: Result<SubmitNonceResonse, FetchError>| {
                match result {
                    Ok(result) => {
                        if d != result.deadline {
                            print!("\r");
                            error!(
                                "submit: deadlines mismatch, height={}, account={}, nonce={}, \
                                 deadline_miner={}, deadline_pool={}",
                                height, account_id, nonce, d, result.deadline
                            );
                        }
                    }
                    Err(FetchError::Pool(e)) => {
                        print!("\r");
                        error!(
                            "submit: error submitting nonce, height={}, account={}, nonce={}, \
                             deadline={}\n\tcode: {}\n\tmessage: {}",
                            height, account_id, nonce, d, e.code, e.message,
                        );
                    }
                    Err(_) => {
                        print!("\r");
                        warn!("submit: error submitting nonce, retry={}", retried,);
                        if retried < 3 {
                            rh.submit_nonce(
                                &inner_handle,
                                account_id,
                                nonce,
                                height,
                                d,
                                retried + 1,
                            );
                        } else {
                            print!("\r");
                            error!("submit: error submitting nonce, exhausted retries");
                        }
                    }
                };
                future::ok(())
            },
        ));
    }

    fn uri_for(&self, path: &str) -> hyper::Uri {
        (self.base_uri.clone() + path).parse().unwrap()
    }

    fn post_req(&self, path: &str) -> Request<hyper::Body> {
        Request::post(self.uri_for(path))
            .header("User-Agent", self.ua.to_owned())
            .body(hyper::Body::empty())
            .unwrap()
    }

    fn get_req(&self, path: &str) -> Request<hyper::Body> {
        Request::get(self.uri_for(path))
            .header("User-Agent", self.ua.to_owned())
            .body(hyper::Body::empty())
            .unwrap()
    }

    fn do_req<T: DeserializeOwned>(
        &self,
        req: Request<hyper::Body>,
    ) -> impl Future<Item = T, Error = FetchError> {
        let req = self
            .client
            .request(req)
            .and_then(|res| res.into_body().concat2())
            .from_err::<FetchError>()
            .and_then(|body| {
                let res = parse_json_result(&body)?;
                Ok(res)
            }).from_err();

        let timeout = Timeout::new(self.timeout, &self.handle).unwrap();
        let timeout = timeout
            .then(|_| Err(io::Error::new(io::ErrorKind::TimedOut, "timeout")))
            .from_err();

        req.select(timeout).then(|res| match res {
            Err((x, _)) => Err(x),
            Ok((x, _)) => Ok(x),
        })
    }
}

fn parse_json_result<T: DeserializeOwned>(c: &hyper::Chunk) -> Result<T, PoolError> {
    match serde_json::from_slice(c) {
        Ok(x) => Ok(x),
        _ => match serde_json::from_slice::<PoolErrorWrapper>(c) {
            Ok(x) => Err(x.error),
            _ => {
                let v = c.to_vec();
                Err(PoolError {
                    code: 0,
                    message: String::from_utf8_lossy(&v).to_string(),
                })
            }
        },
    }
}

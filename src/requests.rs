extern crate hyper;
extern crate serde_json;
extern crate url;

use futures::future;
use hyper::client::HttpConnector;
use hyper::rt::{Future, Stream};
use hyper::{Client, Request};
use serde::de::{self, DeserializeOwned};
use std::fmt;
use std::u64;
use tokio_core::reactor::Handle;
use url::form_urlencoded::byte_serialize;

#[derive(Clone)]
pub struct RequestHandler {
    secret_phrase_encoded: String,
    base_uri: String,
    client: Client<HttpConnector>,
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
    pub fn new(base_uri: String, secret_phrase: String) -> RequestHandler {
        let secret_phrase_encoded = byte_serialize(secret_phrase.as_bytes()).collect();
        RequestHandler {
            secret_phrase_encoded: secret_phrase_encoded,
            base_uri: base_uri,
            client: Client::new(),
        }
    }

    pub fn get_mining_info(&self) -> Box<Future<Item = MiningInfo, Error = FetchError>> {
        let url = (self.base_uri.clone() + &"/burst?requestType=getMiningInfo".to_string())
            .parse()
            .unwrap();
        Box::new(self.get_json(url))
    }

    pub fn submit_nonce(
        &self,
        handle: Handle,
        account_id: u64,
        nonce: u64,
        height: u64,
        d: u64,
        retried: i32,
    ) {
        let mut url = self.base_uri.clone() +
            &format!("/burst?requestType=submitNonce&accountId={}&nonce={}&secretPhrase={}&blockheight={}",
                     account_id, nonce, self.secret_phrase_encoded, height);
        // if pool mining also send the deadline (usefull for proxies)
        if self.secret_phrase_encoded == "" {
            url += &format!("&deadline={}", d);
        }
        let url = url.parse().unwrap();
        let rh = self.clone();
        let inner_handle = handle.clone();
        handle.spawn(self.post_json(url).then(
            move |result: Result<SubmitNonceResonse, FetchError>| {
                match result {
                    Ok(result) => {
                        if d != result.deadline {
                            eprintln!("deadlines mismatch, miner: {} pool: {}", d, result.deadline);
                        }
                    }
                    Err(FetchError::Pool(e)) => {
                        eprintln!(
                            "error submitting nonce:\n\tcode: {}\n\tmessage: {}",
                            e.code, e.message,
                        );
                    }
                    Err(_) => {
                        eprintln!("error submitting nonce:\n\tretry: {}", retried,);
                        if retried < 3 {
                            rh.submit_nonce(
                                inner_handle,
                                account_id,
                                nonce,
                                height,
                                d,
                                retried + 1,
                            );
                        } else {
                            eprintln!("error submitting nonce, exhausted retries");
                        }
                    }
                };
                future::ok(())
            },
        ));
    }

    fn get_json<T: DeserializeOwned>(
        &self,
        uri: hyper::Uri,
    ) -> impl Future<Item = T, Error = FetchError> {
        self.client
            .get(uri)
            .and_then(|res| res.into_body().concat2())
            .from_err::<FetchError>()
            .and_then(|body| {
                let res = parse_json_result(&body)?;
                Ok(res)
            })
            .from_err()
    }

    /* TODO: solve this in a more generic way
    This should be solvable with generics in a much nicer way. However, learning rust is already
    painful enough.
     */
    fn post_json<T: DeserializeOwned>(
        &self,
        uri: hyper::Uri,
    ) -> impl Future<Item = T, Error = FetchError> {
        let req = Request::post(uri).body(hyper::Body::empty()).unwrap();
        self.client
            .request(req)
            .and_then(|res| res.into_body().concat2())
            .from_err::<FetchError>()
            .and_then(|body| {
                let res = parse_json_result(&body)?;
                Ok(res)
            })
            .from_err()
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
                    message: String::from_utf8_lossy(&v).to_string() ,
                })
            },
        },
    }
}

pub enum FetchError {
    Http(hyper::Error),
    Pool(PoolError),
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

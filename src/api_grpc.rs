// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_API_GET_MINING_INFO: ::grpcio::Method<super::api::Void, super::api::GetMiningInfoResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/api.Api/get_mining_info",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_API_SUBMIT_NONCE: ::grpcio::Method<super::api::SubmitNonceRequest, super::api::SubmitNonceResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/api.Api/submit_nonce",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct ApiClient {
    client: ::grpcio::Client,
}

impl ApiClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        ApiClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn get_mining_info_opt(&self, req: &super::api::Void, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::api::GetMiningInfoResponse> {
        self.client.unary_call(&METHOD_API_GET_MINING_INFO, req, opt)
    }

    pub fn get_mining_info(&self, req: &super::api::Void) -> ::grpcio::Result<super::api::GetMiningInfoResponse> {
        self.get_mining_info_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_mining_info_async_opt(&self, req: &super::api::Void, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::api::GetMiningInfoResponse>> {
        self.client.unary_call_async(&METHOD_API_GET_MINING_INFO, req, opt)
    }

    pub fn get_mining_info_async(&self, req: &super::api::Void) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::api::GetMiningInfoResponse>> {
        self.get_mining_info_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn submit_nonce_opt(&self, req: &super::api::SubmitNonceRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::api::SubmitNonceResponse> {
        self.client.unary_call(&METHOD_API_SUBMIT_NONCE, req, opt)
    }

    pub fn submit_nonce(&self, req: &super::api::SubmitNonceRequest) -> ::grpcio::Result<super::api::SubmitNonceResponse> {
        self.submit_nonce_opt(req, ::grpcio::CallOption::default())
    }

    pub fn submit_nonce_async_opt(&self, req: &super::api::SubmitNonceRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::api::SubmitNonceResponse>> {
        self.client.unary_call_async(&METHOD_API_SUBMIT_NONCE, req, opt)
    }

    pub fn submit_nonce_async(&self, req: &super::api::SubmitNonceRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::api::SubmitNonceResponse>> {
        self.submit_nonce_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Api {
    fn get_mining_info(&mut self, ctx: ::grpcio::RpcContext, req: super::api::Void, sink: ::grpcio::UnarySink<super::api::GetMiningInfoResponse>);
    fn submit_nonce(&mut self, ctx: ::grpcio::RpcContext, req: super::api::SubmitNonceRequest, sink: ::grpcio::UnarySink<super::api::SubmitNonceResponse>);
}

pub fn create_api<S: Api + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_API_GET_MINING_INFO, move |ctx, req, resp| {
        instance.get_mining_info(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_API_SUBMIT_NONCE, move |ctx, req, resp| {
        instance.submit_nonce(ctx, req, resp)
    });
    builder.build()
}

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

const METHOD_REGISTER_REGISTER: ::grpcio::Method<super::registry_proto::RegisterRequest, super::registry_proto::RegisterResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/register.Register/Register",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_REGISTER_REPORT_STATUS: ::grpcio::Method<super::registry_proto::StatusRequest, super::registry_proto::StatusResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/register.Register/ReportStatus",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_REGISTER_RE_REGISTER: ::grpcio::Method<super::registry_proto::ReRegisterRequest, super::registry_proto::ReRegisterResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/register.Register/ReRegister",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

pub struct RegisterClient {
    client: ::grpcio::Client,
}

impl RegisterClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        RegisterClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn register_opt(&self, req: &super::registry_proto::RegisterRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::registry_proto::RegisterResponse> {
        self.client.unary_call(&METHOD_REGISTER_REGISTER, req, opt)
    }

    pub fn register(&self, req: &super::registry_proto::RegisterRequest) -> ::grpcio::Result<super::registry_proto::RegisterResponse> {
        self.register_opt(req, ::grpcio::CallOption::default())
    }

    pub fn register_async_opt(&self, req: &super::registry_proto::RegisterRequest, opt: ::grpcio::CallOption) -> ::grpcio::ClientUnaryReceiver<super::registry_proto::RegisterResponse> {
        self.client.unary_call_async(&METHOD_REGISTER_REGISTER, req, opt)
    }

    pub fn register_async(&self, req: &super::registry_proto::RegisterRequest) -> ::grpcio::ClientUnaryReceiver<super::registry_proto::RegisterResponse> {
        self.register_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn report_status_opt(&self, req: &super::registry_proto::StatusRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::registry_proto::StatusResponse> {
        self.client.unary_call(&METHOD_REGISTER_REPORT_STATUS, req, opt)
    }

    pub fn report_status(&self, req: &super::registry_proto::StatusRequest) -> ::grpcio::Result<super::registry_proto::StatusResponse> {
        self.report_status_opt(req, ::grpcio::CallOption::default())
    }

    pub fn report_status_async_opt(&self, req: &super::registry_proto::StatusRequest, opt: ::grpcio::CallOption) -> ::grpcio::ClientUnaryReceiver<super::registry_proto::StatusResponse> {
        self.client.unary_call_async(&METHOD_REGISTER_REPORT_STATUS, req, opt)
    }

    pub fn report_status_async(&self, req: &super::registry_proto::StatusRequest) -> ::grpcio::ClientUnaryReceiver<super::registry_proto::StatusResponse> {
        self.report_status_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn re_register_opt(&self, req: &super::registry_proto::ReRegisterRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::registry_proto::ReRegisterResponse> {
        self.client.unary_call(&METHOD_REGISTER_RE_REGISTER, req, opt)
    }

    pub fn re_register(&self, req: &super::registry_proto::ReRegisterRequest) -> ::grpcio::Result<super::registry_proto::ReRegisterResponse> {
        self.re_register_opt(req, ::grpcio::CallOption::default())
    }

    pub fn re_register_async_opt(&self, req: &super::registry_proto::ReRegisterRequest, opt: ::grpcio::CallOption) -> ::grpcio::ClientUnaryReceiver<super::registry_proto::ReRegisterResponse> {
        self.client.unary_call_async(&METHOD_REGISTER_RE_REGISTER, req, opt)
    }

    pub fn re_register_async(&self, req: &super::registry_proto::ReRegisterRequest) -> ::grpcio::ClientUnaryReceiver<super::registry_proto::ReRegisterResponse> {
        self.re_register_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Register {
    fn register(&self, ctx: ::grpcio::RpcContext, req: super::registry_proto::RegisterRequest, sink: ::grpcio::UnarySink<super::registry_proto::RegisterResponse>);
    fn report_status(&self, ctx: ::grpcio::RpcContext, req: super::registry_proto::StatusRequest, sink: ::grpcio::UnarySink<super::registry_proto::StatusResponse>);
    fn re_register(&self, ctx: ::grpcio::RpcContext, req: super::registry_proto::ReRegisterRequest, sink: ::grpcio::UnarySink<super::registry_proto::ReRegisterResponse>);
}

pub fn create_register<S: Register + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_REGISTER_REGISTER, move |ctx, req, resp| {
        instance.register(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_REGISTER_REPORT_STATUS, move |ctx, req, resp| {
        instance.report_status(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_REGISTER_RE_REGISTER, move |ctx, req, resp| {
        instance.re_register(ctx, req, resp)
    });
    builder.build()
}

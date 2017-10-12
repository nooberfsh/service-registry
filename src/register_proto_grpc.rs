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

const METHOD_REGISTER_REGISTER: ::grpcio::Method<super::register_proto::RegisterRequest, super::register_proto::RegisterResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/register.Register/Register",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_REGISTER_REPORT_STATUS: ::grpcio::Method<super::register_proto::StatusRequest, super::register_proto::RegisterResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/register.Register/ReportStatus",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_REGISTER_RESUME: ::grpcio::Method<super::register_proto::ResumeRequest, super::register_proto::ResumeResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/register.Register/Resume",
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

    pub fn register_opt(&self, req: super::register_proto::RegisterRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::register_proto::RegisterResponse> {
        self.client.unary_call(&METHOD_REGISTER_REGISTER, req, opt)
    }

    pub fn register(&self, req: super::register_proto::RegisterRequest) -> ::grpcio::Result<super::register_proto::RegisterResponse> {
        self.register_opt(req, ::grpcio::CallOption::default())
    }

    pub fn register_async_opt(&self, req: super::register_proto::RegisterRequest, opt: ::grpcio::CallOption) -> ::grpcio::ClientUnaryReceiver<super::register_proto::RegisterResponse> {
        self.client.unary_call_async(&METHOD_REGISTER_REGISTER, req, opt)
    }

    pub fn register_async(&self, req: super::register_proto::RegisterRequest) -> ::grpcio::ClientUnaryReceiver<super::register_proto::RegisterResponse> {
        self.register_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn report_status_opt(&self, req: super::register_proto::StatusRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::register_proto::RegisterResponse> {
        self.client.unary_call(&METHOD_REGISTER_REPORT_STATUS, req, opt)
    }

    pub fn report_status(&self, req: super::register_proto::StatusRequest) -> ::grpcio::Result<super::register_proto::RegisterResponse> {
        self.report_status_opt(req, ::grpcio::CallOption::default())
    }

    pub fn report_status_async_opt(&self, req: super::register_proto::StatusRequest, opt: ::grpcio::CallOption) -> ::grpcio::ClientUnaryReceiver<super::register_proto::RegisterResponse> {
        self.client.unary_call_async(&METHOD_REGISTER_REPORT_STATUS, req, opt)
    }

    pub fn report_status_async(&self, req: super::register_proto::StatusRequest) -> ::grpcio::ClientUnaryReceiver<super::register_proto::RegisterResponse> {
        self.report_status_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn resume_opt(&self, req: super::register_proto::ResumeRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::register_proto::ResumeResponse> {
        self.client.unary_call(&METHOD_REGISTER_RESUME, req, opt)
    }

    pub fn resume(&self, req: super::register_proto::ResumeRequest) -> ::grpcio::Result<super::register_proto::ResumeResponse> {
        self.resume_opt(req, ::grpcio::CallOption::default())
    }

    pub fn resume_async_opt(&self, req: super::register_proto::ResumeRequest, opt: ::grpcio::CallOption) -> ::grpcio::ClientUnaryReceiver<super::register_proto::ResumeResponse> {
        self.client.unary_call_async(&METHOD_REGISTER_RESUME, req, opt)
    }

    pub fn resume_async(&self, req: super::register_proto::ResumeRequest) -> ::grpcio::ClientUnaryReceiver<super::register_proto::ResumeResponse> {
        self.resume_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Register {
    fn register(&self, ctx: ::grpcio::RpcContext, req: super::register_proto::RegisterRequest, sink: ::grpcio::UnarySink<super::register_proto::RegisterResponse>);
    fn report_status(&self, ctx: ::grpcio::RpcContext, req: super::register_proto::StatusRequest, sink: ::grpcio::UnarySink<super::register_proto::RegisterResponse>);
    fn resume(&self, ctx: ::grpcio::RpcContext, req: super::register_proto::ResumeRequest, sink: ::grpcio::UnarySink<super::register_proto::ResumeResponse>);
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
    builder = builder.add_unary_handler(&METHOD_REGISTER_RESUME, move |ctx, req, resp| {
        instance.resume(ctx, req, resp)
    });
    builder.build()
}

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

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(PartialEq,Clone,Default)]
pub struct RegisterRequest {
    // message fields
    pub service_id: u64,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for RegisterRequest {}

impl RegisterRequest {
    pub fn new() -> RegisterRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static RegisterRequest {
        static mut instance: ::protobuf::lazy::Lazy<RegisterRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const RegisterRequest,
        };
        unsafe {
            instance.get(RegisterRequest::new)
        }
    }

    // uint64 service_id = 1;

    pub fn clear_service_id(&mut self) {
        self.service_id = 0;
    }

    // Param is passed by value, moved
    pub fn set_service_id(&mut self, v: u64) {
        self.service_id = v;
    }

    pub fn get_service_id(&self) -> u64 {
        self.service_id
    }

    fn get_service_id_for_reflect(&self) -> &u64 {
        &self.service_id
    }

    fn mut_service_id_for_reflect(&mut self) -> &mut u64 {
        &mut self.service_id
    }
}

impl ::protobuf::Message for RegisterRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.service_id = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.service_id != 0 {
            my_size += ::protobuf::rt::value_size(1, self.service_id, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.service_id != 0 {
            os.write_uint64(1, self.service_id)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for RegisterRequest {
    fn new() -> RegisterRequest {
        RegisterRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<RegisterRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "service_id",
                    RegisterRequest::get_service_id_for_reflect,
                    RegisterRequest::mut_service_id_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<RegisterRequest>(
                    "RegisterRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for RegisterRequest {
    fn clear(&mut self) {
        self.clear_service_id();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for RegisterRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for RegisterRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct StatusRequest {
    // message fields
    pub heartbeat_succeed: bool,
    pub service_succeed: bool,
    pub session_id: u64,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for StatusRequest {}

impl StatusRequest {
    pub fn new() -> StatusRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static StatusRequest {
        static mut instance: ::protobuf::lazy::Lazy<StatusRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const StatusRequest,
        };
        unsafe {
            instance.get(StatusRequest::new)
        }
    }

    // bool heartbeat_succeed = 1;

    pub fn clear_heartbeat_succeed(&mut self) {
        self.heartbeat_succeed = false;
    }

    // Param is passed by value, moved
    pub fn set_heartbeat_succeed(&mut self, v: bool) {
        self.heartbeat_succeed = v;
    }

    pub fn get_heartbeat_succeed(&self) -> bool {
        self.heartbeat_succeed
    }

    fn get_heartbeat_succeed_for_reflect(&self) -> &bool {
        &self.heartbeat_succeed
    }

    fn mut_heartbeat_succeed_for_reflect(&mut self) -> &mut bool {
        &mut self.heartbeat_succeed
    }

    // bool service_succeed = 2;

    pub fn clear_service_succeed(&mut self) {
        self.service_succeed = false;
    }

    // Param is passed by value, moved
    pub fn set_service_succeed(&mut self, v: bool) {
        self.service_succeed = v;
    }

    pub fn get_service_succeed(&self) -> bool {
        self.service_succeed
    }

    fn get_service_succeed_for_reflect(&self) -> &bool {
        &self.service_succeed
    }

    fn mut_service_succeed_for_reflect(&mut self) -> &mut bool {
        &mut self.service_succeed
    }

    // uint64 session_id = 3;

    pub fn clear_session_id(&mut self) {
        self.session_id = 0;
    }

    // Param is passed by value, moved
    pub fn set_session_id(&mut self, v: u64) {
        self.session_id = v;
    }

    pub fn get_session_id(&self) -> u64 {
        self.session_id
    }

    fn get_session_id_for_reflect(&self) -> &u64 {
        &self.session_id
    }

    fn mut_session_id_for_reflect(&mut self) -> &mut u64 {
        &mut self.session_id
    }
}

impl ::protobuf::Message for StatusRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.heartbeat_succeed = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.service_succeed = tmp;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.session_id = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.heartbeat_succeed != false {
            my_size += 2;
        }
        if self.service_succeed != false {
            my_size += 2;
        }
        if self.session_id != 0 {
            my_size += ::protobuf::rt::value_size(3, self.session_id, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.heartbeat_succeed != false {
            os.write_bool(1, self.heartbeat_succeed)?;
        }
        if self.service_succeed != false {
            os.write_bool(2, self.service_succeed)?;
        }
        if self.session_id != 0 {
            os.write_uint64(3, self.session_id)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for StatusRequest {
    fn new() -> StatusRequest {
        StatusRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<StatusRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "heartbeat_succeed",
                    StatusRequest::get_heartbeat_succeed_for_reflect,
                    StatusRequest::mut_heartbeat_succeed_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "service_succeed",
                    StatusRequest::get_service_succeed_for_reflect,
                    StatusRequest::mut_service_succeed_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "session_id",
                    StatusRequest::get_session_id_for_reflect,
                    StatusRequest::mut_session_id_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<StatusRequest>(
                    "StatusRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for StatusRequest {
    fn clear(&mut self) {
        self.clear_heartbeat_succeed();
        self.clear_service_succeed();
        self.clear_session_id();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for StatusRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for StatusRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct RegisterResponse {
    // message fields
    pub heartbeat_port: u32,
    pub service_port: u32,
    pub session_id: u64,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for RegisterResponse {}

impl RegisterResponse {
    pub fn new() -> RegisterResponse {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static RegisterResponse {
        static mut instance: ::protobuf::lazy::Lazy<RegisterResponse> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const RegisterResponse,
        };
        unsafe {
            instance.get(RegisterResponse::new)
        }
    }

    // uint32 heartbeat_port = 1;

    pub fn clear_heartbeat_port(&mut self) {
        self.heartbeat_port = 0;
    }

    // Param is passed by value, moved
    pub fn set_heartbeat_port(&mut self, v: u32) {
        self.heartbeat_port = v;
    }

    pub fn get_heartbeat_port(&self) -> u32 {
        self.heartbeat_port
    }

    fn get_heartbeat_port_for_reflect(&self) -> &u32 {
        &self.heartbeat_port
    }

    fn mut_heartbeat_port_for_reflect(&mut self) -> &mut u32 {
        &mut self.heartbeat_port
    }

    // uint32 service_port = 2;

    pub fn clear_service_port(&mut self) {
        self.service_port = 0;
    }

    // Param is passed by value, moved
    pub fn set_service_port(&mut self, v: u32) {
        self.service_port = v;
    }

    pub fn get_service_port(&self) -> u32 {
        self.service_port
    }

    fn get_service_port_for_reflect(&self) -> &u32 {
        &self.service_port
    }

    fn mut_service_port_for_reflect(&mut self) -> &mut u32 {
        &mut self.service_port
    }

    // uint64 session_id = 3;

    pub fn clear_session_id(&mut self) {
        self.session_id = 0;
    }

    // Param is passed by value, moved
    pub fn set_session_id(&mut self, v: u64) {
        self.session_id = v;
    }

    pub fn get_session_id(&self) -> u64 {
        self.session_id
    }

    fn get_session_id_for_reflect(&self) -> &u64 {
        &self.session_id
    }

    fn mut_session_id_for_reflect(&mut self) -> &mut u64 {
        &mut self.session_id
    }
}

impl ::protobuf::Message for RegisterResponse {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.heartbeat_port = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.service_port = tmp;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.session_id = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.heartbeat_port != 0 {
            my_size += ::protobuf::rt::value_size(1, self.heartbeat_port, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.service_port != 0 {
            my_size += ::protobuf::rt::value_size(2, self.service_port, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.session_id != 0 {
            my_size += ::protobuf::rt::value_size(3, self.session_id, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.heartbeat_port != 0 {
            os.write_uint32(1, self.heartbeat_port)?;
        }
        if self.service_port != 0 {
            os.write_uint32(2, self.service_port)?;
        }
        if self.session_id != 0 {
            os.write_uint64(3, self.session_id)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for RegisterResponse {
    fn new() -> RegisterResponse {
        RegisterResponse::new()
    }

    fn descriptor_static(_: ::std::option::Option<RegisterResponse>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "heartbeat_port",
                    RegisterResponse::get_heartbeat_port_for_reflect,
                    RegisterResponse::mut_heartbeat_port_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "service_port",
                    RegisterResponse::get_service_port_for_reflect,
                    RegisterResponse::mut_service_port_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "session_id",
                    RegisterResponse::get_session_id_for_reflect,
                    RegisterResponse::mut_session_id_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<RegisterResponse>(
                    "RegisterResponse",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for RegisterResponse {
    fn clear(&mut self) {
        self.clear_heartbeat_port();
        self.clear_service_port();
        self.clear_session_id();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for RegisterResponse {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for RegisterResponse {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ResumeRequest {
    // message fields
    pub heartbeat_port: u32,
    pub service_port: u32,
    pub service_id: u64,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ResumeRequest {}

impl ResumeRequest {
    pub fn new() -> ResumeRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ResumeRequest {
        static mut instance: ::protobuf::lazy::Lazy<ResumeRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ResumeRequest,
        };
        unsafe {
            instance.get(ResumeRequest::new)
        }
    }

    // uint32 heartbeat_port = 1;

    pub fn clear_heartbeat_port(&mut self) {
        self.heartbeat_port = 0;
    }

    // Param is passed by value, moved
    pub fn set_heartbeat_port(&mut self, v: u32) {
        self.heartbeat_port = v;
    }

    pub fn get_heartbeat_port(&self) -> u32 {
        self.heartbeat_port
    }

    fn get_heartbeat_port_for_reflect(&self) -> &u32 {
        &self.heartbeat_port
    }

    fn mut_heartbeat_port_for_reflect(&mut self) -> &mut u32 {
        &mut self.heartbeat_port
    }

    // uint32 service_port = 2;

    pub fn clear_service_port(&mut self) {
        self.service_port = 0;
    }

    // Param is passed by value, moved
    pub fn set_service_port(&mut self, v: u32) {
        self.service_port = v;
    }

    pub fn get_service_port(&self) -> u32 {
        self.service_port
    }

    fn get_service_port_for_reflect(&self) -> &u32 {
        &self.service_port
    }

    fn mut_service_port_for_reflect(&mut self) -> &mut u32 {
        &mut self.service_port
    }

    // uint64 service_id = 3;

    pub fn clear_service_id(&mut self) {
        self.service_id = 0;
    }

    // Param is passed by value, moved
    pub fn set_service_id(&mut self, v: u64) {
        self.service_id = v;
    }

    pub fn get_service_id(&self) -> u64 {
        self.service_id
    }

    fn get_service_id_for_reflect(&self) -> &u64 {
        &self.service_id
    }

    fn mut_service_id_for_reflect(&mut self) -> &mut u64 {
        &mut self.service_id
    }
}

impl ::protobuf::Message for ResumeRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.heartbeat_port = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.service_port = tmp;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.service_id = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.heartbeat_port != 0 {
            my_size += ::protobuf::rt::value_size(1, self.heartbeat_port, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.service_port != 0 {
            my_size += ::protobuf::rt::value_size(2, self.service_port, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.service_id != 0 {
            my_size += ::protobuf::rt::value_size(3, self.service_id, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.heartbeat_port != 0 {
            os.write_uint32(1, self.heartbeat_port)?;
        }
        if self.service_port != 0 {
            os.write_uint32(2, self.service_port)?;
        }
        if self.service_id != 0 {
            os.write_uint64(3, self.service_id)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for ResumeRequest {
    fn new() -> ResumeRequest {
        ResumeRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<ResumeRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "heartbeat_port",
                    ResumeRequest::get_heartbeat_port_for_reflect,
                    ResumeRequest::mut_heartbeat_port_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "service_port",
                    ResumeRequest::get_service_port_for_reflect,
                    ResumeRequest::mut_service_port_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "service_id",
                    ResumeRequest::get_service_id_for_reflect,
                    ResumeRequest::mut_service_id_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ResumeRequest>(
                    "ResumeRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ResumeRequest {
    fn clear(&mut self) {
        self.clear_heartbeat_port();
        self.clear_service_port();
        self.clear_service_id();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ResumeRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ResumeRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ResumeResponse {
    // message fields
    pub succeed: bool,
    pub msg: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ResumeResponse {}

impl ResumeResponse {
    pub fn new() -> ResumeResponse {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ResumeResponse {
        static mut instance: ::protobuf::lazy::Lazy<ResumeResponse> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ResumeResponse,
        };
        unsafe {
            instance.get(ResumeResponse::new)
        }
    }

    // bool succeed = 1;

    pub fn clear_succeed(&mut self) {
        self.succeed = false;
    }

    // Param is passed by value, moved
    pub fn set_succeed(&mut self, v: bool) {
        self.succeed = v;
    }

    pub fn get_succeed(&self) -> bool {
        self.succeed
    }

    fn get_succeed_for_reflect(&self) -> &bool {
        &self.succeed
    }

    fn mut_succeed_for_reflect(&mut self) -> &mut bool {
        &mut self.succeed
    }

    // string msg = 2;

    pub fn clear_msg(&mut self) {
        self.msg.clear();
    }

    // Param is passed by value, moved
    pub fn set_msg(&mut self, v: ::std::string::String) {
        self.msg = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_msg(&mut self) -> &mut ::std::string::String {
        &mut self.msg
    }

    // Take field
    pub fn take_msg(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.msg, ::std::string::String::new())
    }

    pub fn get_msg(&self) -> &str {
        &self.msg
    }

    fn get_msg_for_reflect(&self) -> &::std::string::String {
        &self.msg
    }

    fn mut_msg_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.msg
    }
}

impl ::protobuf::Message for ResumeResponse {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.succeed = tmp;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.msg)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.succeed != false {
            my_size += 2;
        }
        if !self.msg.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.msg);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.succeed != false {
            os.write_bool(1, self.succeed)?;
        }
        if !self.msg.is_empty() {
            os.write_string(2, &self.msg)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for ResumeResponse {
    fn new() -> ResumeResponse {
        ResumeResponse::new()
    }

    fn descriptor_static(_: ::std::option::Option<ResumeResponse>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "succeed",
                    ResumeResponse::get_succeed_for_reflect,
                    ResumeResponse::mut_succeed_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "msg",
                    ResumeResponse::get_msg_for_reflect,
                    ResumeResponse::mut_msg_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ResumeResponse>(
                    "ResumeResponse",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ResumeResponse {
    fn clear(&mut self) {
        self.clear_succeed();
        self.clear_msg();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ResumeResponse {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ResumeResponse {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x1aproto/registry_proto.proto\x12\x08register\"0\n\x0fRegisterRequest\
    \x12\x1d\n\nservice_id\x18\x01\x20\x01(\x04R\tserviceId\"\x84\x01\n\rSta\
    tusRequest\x12+\n\x11heartbeat_succeed\x18\x01\x20\x01(\x08R\x10heartbea\
    tSucceed\x12'\n\x0fservice_succeed\x18\x02\x20\x01(\x08R\x0eserviceSucce\
    ed\x12\x1d\n\nsession_id\x18\x03\x20\x01(\x04R\tsessionId\"{\n\x10Regist\
    erResponse\x12%\n\x0eheartbeat_port\x18\x01\x20\x01(\rR\rheartbeatPort\
    \x12!\n\x0cservice_port\x18\x02\x20\x01(\rR\x0bservicePort\x12\x1d\n\nse\
    ssion_id\x18\x03\x20\x01(\x04R\tsessionId\"x\n\rResumeRequest\x12%\n\x0e\
    heartbeat_port\x18\x01\x20\x01(\rR\rheartbeatPort\x12!\n\x0cservice_port\
    \x18\x02\x20\x01(\rR\x0bservicePort\x12\x1d\n\nservice_id\x18\x03\x20\
    \x01(\x04R\tserviceId\"<\n\x0eResumeResponse\x12\x18\n\x07succeed\x18\
    \x01\x20\x01(\x08R\x07succeed\x12\x10\n\x03msg\x18\x02\x20\x01(\tR\x03ms\
    g2\xd5\x01\n\x08Register\x12C\n\x08Register\x12\x19.register.RegisterReq\
    uest\x1a\x1a.register.RegisterResponse\"\0\x12E\n\x0cReportStatus\x12\
    \x17.register.StatusRequest\x1a\x1a.register.RegisterResponse\"\0\x12=\n\
    \x06Resume\x12\x17.register.ResumeRequest\x1a\x18.register.ResumeRespons\
    e\"\0J\x99\t\n\x06\x12\x04\0\0#\x01\n\x08\n\x01\x0c\x12\x03\0\0\x12\n\
    \x08\n\x01\x02\x12\x03\x02\x08\x10\n\n\n\x02\x04\0\x12\x04\x04\0\x06\x01\
    \n\n\n\x03\x04\0\x01\x12\x03\x04\x08\x17\n\x0b\n\x04\x04\0\x02\0\x12\x03\
    \x05\x08\x1e\n\r\n\x05\x04\0\x02\0\x04\x12\x04\x05\x08\x04\x19\n\x0c\n\
    \x05\x04\0\x02\0\x05\x12\x03\x05\x08\x0e\n\x0c\n\x05\x04\0\x02\0\x01\x12\
    \x03\x05\x0f\x19\n\x0c\n\x05\x04\0\x02\0\x03\x12\x03\x05\x1c\x1d\n\n\n\
    \x02\x04\x01\x12\x04\x08\0\x0c\x01\n\n\n\x03\x04\x01\x01\x12\x03\x08\x08\
    \x15\n\x0b\n\x04\x04\x01\x02\0\x12\x03\t\x08#\n\r\n\x05\x04\x01\x02\0\
    \x04\x12\x04\t\x08\x08\x17\n\x0c\n\x05\x04\x01\x02\0\x05\x12\x03\t\x08\
    \x0c\n\x0c\n\x05\x04\x01\x02\0\x01\x12\x03\t\r\x1e\n\x0c\n\x05\x04\x01\
    \x02\0\x03\x12\x03\t!\"\n\x0b\n\x04\x04\x01\x02\x01\x12\x03\n\x08!\n\r\n\
    \x05\x04\x01\x02\x01\x04\x12\x04\n\x08\t#\n\x0c\n\x05\x04\x01\x02\x01\
    \x05\x12\x03\n\x08\x0c\n\x0c\n\x05\x04\x01\x02\x01\x01\x12\x03\n\r\x1c\n\
    \x0c\n\x05\x04\x01\x02\x01\x03\x12\x03\n\x1f\x20\n\x0b\n\x04\x04\x01\x02\
    \x02\x12\x03\x0b\x08\x1e\n\r\n\x05\x04\x01\x02\x02\x04\x12\x04\x0b\x08\n\
    !\n\x0c\n\x05\x04\x01\x02\x02\x05\x12\x03\x0b\x08\x0e\n\x0c\n\x05\x04\
    \x01\x02\x02\x01\x12\x03\x0b\x0f\x19\n\x0c\n\x05\x04\x01\x02\x02\x03\x12\
    \x03\x0b\x1c\x1d\n\n\n\x02\x04\x02\x12\x04\x0e\0\x12\x01\n\n\n\x03\x04\
    \x02\x01\x12\x03\x0e\x08\x18\n\x0b\n\x04\x04\x02\x02\0\x12\x03\x0f\x08\"\
    \n\r\n\x05\x04\x02\x02\0\x04\x12\x04\x0f\x08\x0e\x1a\n\x0c\n\x05\x04\x02\
    \x02\0\x05\x12\x03\x0f\x08\x0e\n\x0c\n\x05\x04\x02\x02\0\x01\x12\x03\x0f\
    \x0f\x1d\n\x0c\n\x05\x04\x02\x02\0\x03\x12\x03\x0f\x20!\n\x0b\n\x04\x04\
    \x02\x02\x01\x12\x03\x10\x08\x20\n\r\n\x05\x04\x02\x02\x01\x04\x12\x04\
    \x10\x08\x0f\"\n\x0c\n\x05\x04\x02\x02\x01\x05\x12\x03\x10\x08\x0e\n\x0c\
    \n\x05\x04\x02\x02\x01\x01\x12\x03\x10\x0f\x1b\n\x0c\n\x05\x04\x02\x02\
    \x01\x03\x12\x03\x10\x1e\x1f\n\x0b\n\x04\x04\x02\x02\x02\x12\x03\x11\x08\
    \x1e\n\r\n\x05\x04\x02\x02\x02\x04\x12\x04\x11\x08\x10\x20\n\x0c\n\x05\
    \x04\x02\x02\x02\x05\x12\x03\x11\x08\x0e\n\x0c\n\x05\x04\x02\x02\x02\x01\
    \x12\x03\x11\x0f\x19\n\x0c\n\x05\x04\x02\x02\x02\x03\x12\x03\x11\x1c\x1d\
    \n\n\n\x02\x04\x03\x12\x04\x14\0\x18\x01\n\n\n\x03\x04\x03\x01\x12\x03\
    \x14\x08\x15\n\x0b\n\x04\x04\x03\x02\0\x12\x03\x15\x08\"\n\r\n\x05\x04\
    \x03\x02\0\x04\x12\x04\x15\x08\x14\x17\n\x0c\n\x05\x04\x03\x02\0\x05\x12\
    \x03\x15\x08\x0e\n\x0c\n\x05\x04\x03\x02\0\x01\x12\x03\x15\x0f\x1d\n\x0c\
    \n\x05\x04\x03\x02\0\x03\x12\x03\x15\x20!\n\x0b\n\x04\x04\x03\x02\x01\
    \x12\x03\x16\x08\x20\n\r\n\x05\x04\x03\x02\x01\x04\x12\x04\x16\x08\x15\"\
    \n\x0c\n\x05\x04\x03\x02\x01\x05\x12\x03\x16\x08\x0e\n\x0c\n\x05\x04\x03\
    \x02\x01\x01\x12\x03\x16\x0f\x1b\n\x0c\n\x05\x04\x03\x02\x01\x03\x12\x03\
    \x16\x1e\x1f\n\x0b\n\x04\x04\x03\x02\x02\x12\x03\x17\x08\x1e\n\r\n\x05\
    \x04\x03\x02\x02\x04\x12\x04\x17\x08\x16\x20\n\x0c\n\x05\x04\x03\x02\x02\
    \x05\x12\x03\x17\x08\x0e\n\x0c\n\x05\x04\x03\x02\x02\x01\x12\x03\x17\x0f\
    \x19\n\x0c\n\x05\x04\x03\x02\x02\x03\x12\x03\x17\x1c\x1d\n\n\n\x02\x04\
    \x04\x12\x04\x1a\0\x1d\x01\n\n\n\x03\x04\x04\x01\x12\x03\x1a\x08\x16\n\
    \x0b\n\x04\x04\x04\x02\0\x12\x03\x1b\x08\x19\n\r\n\x05\x04\x04\x02\0\x04\
    \x12\x04\x1b\x08\x1a\x18\n\x0c\n\x05\x04\x04\x02\0\x05\x12\x03\x1b\x08\
    \x0c\n\x0c\n\x05\x04\x04\x02\0\x01\x12\x03\x1b\r\x14\n\x0c\n\x05\x04\x04\
    \x02\0\x03\x12\x03\x1b\x17\x18\n\x0b\n\x04\x04\x04\x02\x01\x12\x03\x1c\
    \x08\x17\n\r\n\x05\x04\x04\x02\x01\x04\x12\x04\x1c\x08\x1b\x19\n\x0c\n\
    \x05\x04\x04\x02\x01\x05\x12\x03\x1c\x08\x0e\n\x0c\n\x05\x04\x04\x02\x01\
    \x01\x12\x03\x1c\x0f\x12\n\x0c\n\x05\x04\x04\x02\x01\x03\x12\x03\x1c\x15\
    \x16\n\n\n\x02\x06\0\x12\x04\x1f\0#\x01\n\n\n\x03\x06\0\x01\x12\x03\x1f\
    \x08\x10\n\x0b\n\x04\x06\0\x02\0\x12\x03\x20\x08C\n\x0c\n\x05\x06\0\x02\
    \0\x01\x12\x03\x20\x0c\x14\n\x0c\n\x05\x06\0\x02\0\x02\x12\x03\x20\x15$\
    \n\x0c\n\x05\x06\0\x02\0\x03\x12\x03\x20/?\n\x0b\n\x04\x06\0\x02\x01\x12\
    \x03!\x08E\n\x0c\n\x05\x06\0\x02\x01\x01\x12\x03!\x0c\x18\n\x0c\n\x05\
    \x06\0\x02\x01\x02\x12\x03!\x19&\n\x0c\n\x05\x06\0\x02\x01\x03\x12\x03!1\
    A\n\x0b\n\x04\x06\0\x02\x02\x12\x03\"\x08=\n\x0c\n\x05\x06\0\x02\x02\x01\
    \x12\x03\"\x0c\x12\n\x0c\n\x05\x06\0\x02\x02\x02\x12\x03\"\x13\x20\n\x0c\
    \n\x05\x06\0\x02\x02\x03\x12\x03\"+9b\x06proto3\
";

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}

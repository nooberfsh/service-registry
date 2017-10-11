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
pub struct HeartbeatRequest {
    // message fields
    pub msg: u32,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for HeartbeatRequest {}

impl HeartbeatRequest {
    pub fn new() -> HeartbeatRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static HeartbeatRequest {
        static mut instance: ::protobuf::lazy::Lazy<HeartbeatRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const HeartbeatRequest,
        };
        unsafe {
            instance.get(HeartbeatRequest::new)
        }
    }

    // uint32 msg = 1;

    pub fn clear_msg(&mut self) {
        self.msg = 0;
    }

    // Param is passed by value, moved
    pub fn set_msg(&mut self, v: u32) {
        self.msg = v;
    }

    pub fn get_msg(&self) -> u32 {
        self.msg
    }

    fn get_msg_for_reflect(&self) -> &u32 {
        &self.msg
    }

    fn mut_msg_for_reflect(&mut self) -> &mut u32 {
        &mut self.msg
    }
}

impl ::protobuf::Message for HeartbeatRequest {
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
                    self.msg = tmp;
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
        if self.msg != 0 {
            my_size += ::protobuf::rt::value_size(1, self.msg, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.msg != 0 {
            os.write_uint32(1, self.msg)?;
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

impl ::protobuf::MessageStatic for HeartbeatRequest {
    fn new() -> HeartbeatRequest {
        HeartbeatRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<HeartbeatRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "msg",
                    HeartbeatRequest::get_msg_for_reflect,
                    HeartbeatRequest::mut_msg_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<HeartbeatRequest>(
                    "HeartbeatRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for HeartbeatRequest {
    fn clear(&mut self) {
        self.clear_msg();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for HeartbeatRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for HeartbeatRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct HeartbeatResponse {
    // message fields
    pub msg: u32,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for HeartbeatResponse {}

impl HeartbeatResponse {
    pub fn new() -> HeartbeatResponse {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static HeartbeatResponse {
        static mut instance: ::protobuf::lazy::Lazy<HeartbeatResponse> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const HeartbeatResponse,
        };
        unsafe {
            instance.get(HeartbeatResponse::new)
        }
    }

    // uint32 msg = 1;

    pub fn clear_msg(&mut self) {
        self.msg = 0;
    }

    // Param is passed by value, moved
    pub fn set_msg(&mut self, v: u32) {
        self.msg = v;
    }

    pub fn get_msg(&self) -> u32 {
        self.msg
    }

    fn get_msg_for_reflect(&self) -> &u32 {
        &self.msg
    }

    fn mut_msg_for_reflect(&mut self) -> &mut u32 {
        &mut self.msg
    }
}

impl ::protobuf::Message for HeartbeatResponse {
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
                    self.msg = tmp;
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
        if self.msg != 0 {
            my_size += ::protobuf::rt::value_size(1, self.msg, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.msg != 0 {
            os.write_uint32(1, self.msg)?;
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

impl ::protobuf::MessageStatic for HeartbeatResponse {
    fn new() -> HeartbeatResponse {
        HeartbeatResponse::new()
    }

    fn descriptor_static(_: ::std::option::Option<HeartbeatResponse>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "msg",
                    HeartbeatResponse::get_msg_for_reflect,
                    HeartbeatResponse::mut_msg_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<HeartbeatResponse>(
                    "HeartbeatResponse",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for HeartbeatResponse {
    fn clear(&mut self) {
        self.clear_msg();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for HeartbeatResponse {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for HeartbeatResponse {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x15heartbeat_proto.proto\x12\theartbeat\"$\n\x10HeartbeatRequest\x12\
    \x10\n\x03msg\x18\x01\x20\x01(\rR\x03msg\"%\n\x11HeartbeatResponse\x12\
    \x10\n\x03msg\x18\x01\x20\x01(\rR\x03msgJ\xd8\x01\n\x06\x12\x04\0\0\n\
    \x01\n\x08\n\x01\x0c\x12\x03\0\0\x12\n\x08\n\x01\x02\x12\x03\x02\x08\x11\
    \n\n\n\x02\x04\0\x12\x04\x04\0\x06\x01\n\n\n\x03\x04\0\x01\x12\x03\x04\
    \x08\x18\n\x0b\n\x04\x04\0\x02\0\x12\x03\x05\x08\x17\n\r\n\x05\x04\0\x02\
    \0\x04\x12\x04\x05\x08\x04\x1a\n\x0c\n\x05\x04\0\x02\0\x05\x12\x03\x05\
    \x08\x0e\n\x0c\n\x05\x04\0\x02\0\x01\x12\x03\x05\x0f\x12\n\x0c\n\x05\x04\
    \0\x02\0\x03\x12\x03\x05\x15\x16\n\n\n\x02\x04\x01\x12\x04\x08\0\n\x01\n\
    \n\n\x03\x04\x01\x01\x12\x03\x08\x08\x19\n\x0b\n\x04\x04\x01\x02\0\x12\
    \x03\t\x08\x17\n\r\n\x05\x04\x01\x02\0\x04\x12\x04\t\x08\x08\x1b\n\x0c\n\
    \x05\x04\x01\x02\0\x05\x12\x03\t\x08\x0e\n\x0c\n\x05\x04\x01\x02\0\x01\
    \x12\x03\t\x0f\x12\n\x0c\n\x05\x04\x01\x02\0\x03\x12\x03\t\x15\x16b\x06p\
    roto3\
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

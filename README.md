# service-registry

generate sources
----------------

```
$ cargo install protobuf
$ cargo install grpcio-compiler
$ protoc --rust_out=. --grpc_out=. --plugin=protoc-gen-grpc=`which grpc_rust_plugin` example.proto
```

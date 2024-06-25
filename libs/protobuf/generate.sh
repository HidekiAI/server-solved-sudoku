#!/bin/bash
_TARGET=/dev/shm/protobuf/generated/
if ! [ -e $_TARGET ]; then
    mkdir -p $_TARGET
fi

# build RPC service via plugins
protoc  --grpc_out=$_TARGET -I .  --plugin=protoc-gen-grpc=`which grpc_cpp_plugin` *.proto
# build messages
protoc --cpp_out=$_TARGET -I .  *.proto

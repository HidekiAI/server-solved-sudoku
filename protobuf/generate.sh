#!/bin/bash
if ! [ -e generated ]; then
    mkdir generated
fi

# build RPC service via plugins
protoc  --grpc_out=generated/ -I .  --plugin=protoc-gen-grpc=`which grpc_cpp_plugin` *.proto
# build messages
protoc --cpp_out=generated/ -I .  *.proto

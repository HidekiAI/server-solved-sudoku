#!/bin/bash
if ! [ -e generated ]; then
    mkdir generated
fi

protoc --cpp_out=generated/ --experimental_allow_proto3_optional *.proto

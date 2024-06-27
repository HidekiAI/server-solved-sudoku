# Generates:
#  protobuf/sudoku.pb.h
#  protobuf/sudoku.pb.cc
#  protobuf/libsudoku_proto.a
#  protobuf/libsudoku_proto.so
load("@rules_cc//cc:defs.bzl", "cc_proto_library")
cc_proto_library(
    name = "sudoku_cc_proto",
    deps = [":sudoku_proto"],
)

# generates 'protobuf/sudoku_proto-descriptor-set.proto.bin' (which is useless for me, need header file)
load("@rules_proto//proto:defs.bzl", "proto_library")
proto_library(
    name = "sudoku_proto",
    srcs = ["sudoku.proto"],
    deps = [],
    visibility = ["//visibility:public"]
)

load("@rules_proto_grpc//cpp:defs.bzl", "cpp_proto_library")
#cpp_proto_library(
#    name = "cpp_sudoku_proto",
#    protos = [":sudoku_proto"],
#)

# Generates 'protobuf/libsudoku_grpc.so' (-lsudoku_grpc.so or -lsudoku_grpc.a)
load("@rules_proto_grpc//cpp:defs.bzl", "cpp_grpc_library")
cpp_grpc_library(
    name = "sudoku_grpc",
    protos = [":sudoku_proto"],
    #deps = [":sudoku_proto"],
    visibility = ["//visibility:public"]
)

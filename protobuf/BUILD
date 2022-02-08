# Generates:
#  protobuf/route.pb.h
#  protobuf/route.pb.cc
#  protobuf/libroute_proto.a
#  protobuf/libroute_proto.so
load("@rules_cc//cc:defs.bzl", "cc_proto_library")
cc_proto_library(
    name = "route_cc_proto",
    deps = [":route_proto"],
)

# generates 'protobuf/route_proto-descriptor-set.proto.bin' (which is useless for me, need header file)
load("@rules_proto//proto:defs.bzl", "proto_library")
proto_library(
    name = "route_proto",
    srcs = ["route.proto"],
    deps = [],
    visibility = ["//visibility:public"]
)

load("@rules_proto_grpc//cpp:defs.bzl", "cpp_proto_library")
#cpp_proto_library(
#    name = "cpp_route_proto",
#    protos = [":route_proto"],
#)

# Generates 'protobuf/libroute_grpc.so' (-lroute_grpc.so or -lroute_grpc.a)
load("@rules_proto_grpc//cpp:defs.bzl", "cpp_grpc_library")
cpp_grpc_library(
    name = "route_grpc",
    protos = [":route_proto"],
    #deps = [":route_proto"],
    visibility = ["//visibility:public"]
)

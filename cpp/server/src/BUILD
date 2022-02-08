cc_import(
  name = "route_grpc",
  hdrs = ["protobuf/route.pb.h"],
  static_library = "protobuf/libroute_grpc.a",
  shared_library = "protobuf/libroute_grpc.so",
)

cc_library (
    name = "solver",
    srcs = ["solver.cpp"],
    hdrs = ["solver.hpp", "data_models.hpp"],
    includes = ["./"],
    deps = ["//protobuf:route_grpc"],
)
# C++ implementations of server solver

## Some Design thoughts

* Parallel multithread evaluator of each 3x3 grid to determine probable values
* Multithread version - distribitor to use std::lock_guard on the 3x3 grid
* Multi-process version - usage of gRPC and protobuf (other pub-sub methods using MQ is an overkill, but possibly consider zeroMQ+gRPC combo)


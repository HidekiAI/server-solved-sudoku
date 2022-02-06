#include <iostream>
#include <memory>
#include <thread>

// Debian libgrpc++-dev package version (source tracks back to from github.com/grpc homepage: grpc.io)
#include <grpc/support/log.h>
#include <grpcpp/grpcpp.h>

#include "../../protobuf/generated/route.pb.h"
#include "data_models.hpp"

namespace Sudoku
{
    class Solver final
    {
        public:
            Solver();
    };
}
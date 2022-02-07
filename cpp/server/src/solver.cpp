#include "solver.hpp"

#include <iostream>
#include <memory>
#include <string>
#include <thread>
#include <optional>

#include <grpcpp/grpcpp.h>
#include <grpc/support/log.h>
#include <grpcpp/server.h>


#include "../../protobuf/generated/route.grpc.pb.h"

using grpc::Server;
using grpc::ServerAsyncResponseWriter;
using grpc::ServerBuilder;
using grpc::ServerCompletionQueue;
using grpc::ServerContext;
using grpc::Status;
using Router::Route;

namespace Sudoku
{
    Solver::Solver()
    {
        CompletionQueue cq;
        std::unique_ptr<ClientAsyncResponseReader<Router::Game> > rpc( stub_->AsyncSayHello(&context, request, &cq));

        auto myCell = DataModels::Cell<int64_t>(std::nullopt, std::nullopt, std::vector<int64_t>(1));
    }
}
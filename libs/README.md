# Servers

Servers are docker-based (and any dependencies are tied via docker-compose) and is 100% Rust.

Though it feels (definitely is) overdesigned and overkill, as a practice, the generator microservice, solver/evaluator microservice, and game microservice are split into 3 separate docker containers, and are communicated via gRPC.

## Client simulator vs Unit-test

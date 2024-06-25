# Server Solved Sudoku

A server-client based sudoku where solvers are configurable and resides on the (over-engineered) [server](#server) side.  S2S and C2S are all (at the time of the design) considered to be [gRPC](https://en.wikipedia.org/wiki/GRPC) based.

There are plenty of sudoku solvers and applications on github, so if you're looking for application-game to just play sudoku, this is not the app.  This is basically a demonstrations of how a scalable game server in which clients can be of any kind (headless, GUI, TUI, CLI+ML, etc).

There are few choices of system-engineering made that could go in different implementations but does the same that needed to be decide at the get-go.  These are:

- gRPC versus REST - though I could still go hybrid and choose S2S to be gRPC but C2S to be REST-based, at this time, I'm going to stick with gRPC for C2S.  Perhaps it will be easy (in design) to switch to REST in the future (even between S2S) but one will realize it'll take several man-hours to convert the communication protocol.  Though it should still remain in mind mainly because at the C2S level, if I was to go with REST-based, rather than relying on Google load-balancer to proxy and route client requests/responses, I can use Nginx, Apache, or IIS to do the routing.  So do keep in mind about this, and do try to decide on this early.  Though some will argue that there are more flexibility in REST because the client can be of almost any languages since REST has been around since the invention of HTTP, but then again, I argue that [Protobuf](https://en.wikipedia.org/wiki/Protocol_Buffers) 3 for gRPC has been around long enough (7 years?), and clients such as Unity (mono-C#), Unreal (C++), Godot (C++ and/or mono-C#) could easily adopt to gRPC.  Sure, writing BASH script using [nc](https://en.wikipedia.org/wiki/Netcat) or [cURL](https://en.wikipedia.org/wiki/CURL) against REST protocol is much more convinient than gRPC but well, these days, I tend to write in Rust whenever BASH script starts looking bit too complicated...  In any case, I'll keep in mind, but I'm sticking with gRPC for now, and probably will stick with it.
- [Kafka](https://en.wikipedia.org/wiki/Apache_Kafka) vs the rest/others ([Redis](https://en.wikipedia.org/wiki/Redis), [RabbitMQ](https://en.wikipedia.org/wiki/RabbitMQ), etc) - Again, for this project, it is over-engineered to even use Kafka.  I can probably get away with rolling my own something that is similar to [Boost.Signals2](https://www.boost.org/doc/libs/1_74_0/doc/html/signals2.html) usage of pub-sub model.  But in the end, for this project, I want to just use Kafka just-because...  Again, I'll try to be careful to be able to swap Kafka with something else in the future, but well, for this design, I've decided on Kafka one morning, and will stick with it.

Over all, if I was to wear a System Engineer's hat (I'm a game service engineer by trade, so please do not discount me), these are few things that needs to be decided in the initial design (or at least as early as possible, i.e. do a prototype and prove it works), agreed by all engineers (in this case, just me) and discuss about caveats and fallback plans if _X_ doesn't work well, we'll eat _Y_ weeks to switch to _Z_, etc.  I think in this case, the worst is probably being told "we want to use IIS on Azure, so we want to host your services on Windows server, which means we need clients to speak REST and server to receive REST" on say 3 months before release date...

## Libraries

All and any shared logics, data-models (structs, objects, protobuf data) are all in Rust, and resides  in the `lib` folder.  This is to ensure that the server and client can share the same logic and data-models, and to ensure that the server and client can be built and tested independently of each other, yet if data-model (more specific, protobuf data-contract) changes, both server and client can be rebuilt and tested together to ensure that they are compatible with each other.

There will be no versioning, in which if lib changes/updates, both server and client will require rebuilding.

## Server

[Servers](server/README.md) are docker-based (and any dependencies are tied via docker-compose) and is 100% Rust.

Though it feels (definitely is) overdesigned and overkill, as a practice, the generator microservice, solver/evaluator microservice, and game microservice are split into 3 separate docker containers, and are communicated via gRPC.  Though it will be very simple to make a monolithic server by combining all 3 modules into one, and there is nothing wrong on such design as long as it doesn't need to be scaled, as mentioned, this is a practice and my past experiences with monolithic servers attempting to become micro-service has been ... not so good.  So I'm trying to stay away from that pattern now a days...  In fact, if I was going to make the service monolithic, I'd rather just completely toss the need for server, and embed the whole logic into the client.

More details on its designs are discussed  [here](libs/README.md).

Side note: Recently, I had a job-interview where the CEO of such indie game company were looking for a backend engineer, and from what I was told, they had hired frontend engineers to make a network game, and then, as a server, they made the client to be able to play headless, and called it "server".  The CEO was explaining to me that he wanted to have many players play against each other, and the horror thought crossed my mind of how painful it will be to strip out this headless client as a docker image to instance just 2 players (PvP), in which if there are 100 players, it'll mean I'd need to instance 50 docker containers just to play against each other (all because each headless client can only handle serving one game).  It could be done I'd imagine, if that headless client that played the game were made stateless, so that all we had to do was write a micro-service which pulled the current state of the game (of 2 clients) from some persisted storage (i.e. maybe Kafka), stored/updated the new states of each clients back to persisted storage.  Then, basically you can have some routing service (nginx, apache, IIS, or even gcloud load-balancer) route maybe up to 2000 clients per docker container, etc.  I wasn't confident enough to answer how much time it will take to do all this, it could be trivial and quick, or it can be forever maintenanced never-ending project.  I don't know...

## Client

Client (at the time of the writing) are [TUI](client_tui/README.md) based and [ML Trainer](client_ai/README.md).  

Though TUI can probably be used in conjunctions with RL but in general, it's only there to play game normally and no more.  There is a BASH version of Sudoku (quite impressive, all written in bash-script!) which even does annotations!  But my version of TUI is not even going to be that sophisticated.

The ML Trainer client is head-less, but what makes it useful is that it will make me constantly think about what data are needed for ML training as its inputs to solve each cell.  Though truthfully, all ML needs is current state and no more.  Current state is basically a 9x9 matrix (81 cells) - single dimensional array, and solver needs to be called (at worst case) 80 times to return probability of what the digit should be set on the empty cell (which means, at worst case, each cell will have 9 possible answers, so that would be 9x9x9 = 6561 possible answers for the entire board).  Read more about the ML version [here](libs/README.md) on both the server and [client](client_ai/README.md) side.

#   # We will first need to lib due to dependencies, hence we'll be doing multi-stage build
#   # 1. Build libscsudoku - assumes that make-service.sh shell script has copied libs to "build/libs" directory
#   # 2. Build oauth_relay_service
#   # Reminder: even though I'd name it as /user/src/scsudoku on BOTH stages, it's actually different directories
#   # in which neither of the two stages can see each other's files.
#   
#   ##################################### stage 1
#   # Stage 1: libscsudoku
#   FROM rust:latest as builder-lib-stage
#   WORKDIR /usr/src/scsudoku/libs
#   COPY ./build/libs ./
#   RUN ls -lAhr
#   # we mainly need protoc
#   RUN apt-get update                                      \
#       && apt-get update --fix-missing                     \
#       && apt-get install -y --install-recommends          \
#           apt-utils curl zip unzip tar git                \
#       && apt-get install -y --install-recommends          \
#           gcc g++                                         \
#           libclang-dev libc-devtools libc-dev-bin make    \
#       && apt-get install -y --install-recommends          \
#           librust-vcpkg-dev cmake clang                   \
#           pkg-config                                      \
#       && gcc --version && g++ --version
#   
#   # Install vcpkg
#   RUN git clone https://github.com/microsoft/vcpkg.git /opt/vcpkg && \
#       /opt/vcpkg/bootstrap-vcpkg.sh
#   ENV PATH="/opt/vcpkg:${PATH}"
#   
#   RUN apt-get install -y --install-recommends             \
#           protobuf-codegen grpc-proto protobuf-compiler   \
#       && apt-get install -y --install-recommends          \
#           ca-certificates tzdata libssl-dev               \
#       && apt-get autoclean && apt-get autoremove && rm -rf /var/cache/apt/archives/*
#   RUN cargo build --release
#   
#   ##################################### stage 2
#   # Stage 2: Build the service oauth_relay_service
#   
#   # Use the official Rust image as the base image
#   FROM rust:latest
#   
#   # Set the working directory inside the container
#   WORKDIR /usr/src/scsudoku/oauth_relay_service
#   
#   # Copy build/libs so that Cargo.toml can use dpendencies of 'libscsudoku=./build/libs' directory
#   COPY build ./build
#   
#   # Copy output from the first stage to the current stage
#   COPY --from=builder-lib-stage /usr/src/scsudoku/libs/target/release/* ./target/release/
#   
#   # Copy the Cargo.toml and Cargo.lock files
#   COPY Cargo.* ./
#   
#   # Copy the source code
#   COPY src ./src
#   
#   # Copy the .env file - used by Config
#   COPY .env ./.env
#   COPY .env.local ./.env.local
#   
#   RUN whoami
#   
#   # Install oauth_relay_services dependant by 3rd party crates
#   RUN apt-get update                                      \
#       && apt-get update --fix-missing                     \
#       && apt-get install -y --install-recommends          \
#           apt-utils curl zip unzip tar git                \
#       && apt-get install -y --install-recommends          \
#           gcc g++                                         \
#           libclang-dev libc-devtools libc-dev-bin make    \
#       && apt-get install -y --install-recommends          \
#           librust-vcpkg-dev cmake clang                   \
#           pkg-config                                      \
#       && gcc --version && g++ --version
#   
#   # Install vcpkg
#   RUN git clone https://github.com/microsoft/vcpkg.git /opt/vcpkg && \
#       /opt/vcpkg/bootstrap-vcpkg.sh
#   ENV PATH="/opt/vcpkg:${PATH}"
#   # NOTE: On MinGW64, you `vcpkg install librdkafka:x64-mingw-dynamic`
#   #       On Linux, you `vcpkg install librdkafka:x64-linux`
#   #       On Windows, you `vcpkg install librdkafka:x64-windows` BUT without VStudios installed, it WILL FAIL, so use mingw instead!
#   RUN vcpkg install librdkafka:x64-linux
#   
#   RUN apt-get install -y --install-recommends             \
#           protobuf-codegen grpc-proto protobuf-compiler   \
#       && apt-get install -y --install-recommends          \
#           ca-certificates tzdata libssl-dev               \
#           libsqlite3-dev                                  \
#           librdkafka-dev                                  \
#       && apt-get autoclean && apt-get autoremove && rm -rf /var/cache/apt/archives/*
#   
#   # Build the Rust oauth_relay_servicelication (NOTE: We assume prior to 'Docker build' this project, all 'cargo build --release' was done and copied)
#   RUN cargo build --release

# NOTE: Original approach was to use multi-stage build, but it was too complex to maintain
#       due to Cargo.toml dependencies wanting directories above the project directory.
#       i.e.  'libscsudoku = { path ="../libs" }'
#       Hence, we'll ASSUME that the entire micro-services are built outside
#       this project at the project-root level and binaries reside in '$ROOTDIR/target/release'
#       in which the BASH shell script 'make-service.sh' will copy the binaries to each
#       micro-service's that has "Dockerfile" so that './build/target/release' directory exists
#       side-by-side with the 'Dockerfile'.
# Use the official Rust image as the base image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /usr/src/scsudoku/oauth_relay_service

# Copes build/.env, build.env.local, build/target/release, etc
COPY build ./

# Expose ports for the web server and TCP server
EXPOSE ${REST_PORT}

# Run the compiled binary
CMD ["./build/target/release/oauth_relay_service"]

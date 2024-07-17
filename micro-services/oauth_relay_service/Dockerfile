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

# Copies build/.env, build.env.local, build/target/release, etc
# NOTE: Files that are required for THIS service is explicitly copied, so that
#       if `cargo build --release` had failed, the missing binaries will be
#       detected and the build will fail.
COPY build/oauth_relay_service* ./
COPY build/.env ./
COPY build/.env.local ./
COPY build ./
RUN ls -AR

# Expose ports for the web server and TCP server (see .env* files which set the env-vars)
EXPOSE ${REST_PORT}

# Run the compiled binary
CMD ["./build/target/release/oauth_relay_service"]

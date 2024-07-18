# NOTE: Original approach was to use multi-stage build, but it was too complex to maintain
#       due to Cargo.toml dependencies wanting directories above the project directory.
#       i.e.  'libscsudoku = { path ="../libs" }'
#       Hence, we'll ASSUME that the entire micro-services are built outside
#       this project at the project-root level and binaries reside in '$ROOTDIR/target/release'
#       in which the BASH shell script 'make-service.sh' will copy the binaries to each
#       micro-service's that has "Dockerfile" so that './build/target/release' directory exists
#       side-by-side with the 'Dockerfile'.

FROM alpine as data-copy-stage
WORKDIR /data
# Copy the existing db.sqlite3 file to the directory IF it exists...
COPY ./data/* ./data/

# Use the official Rust image as the base image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /usr/src/scsudoku/oauth_relay_service

# Create the directory so sqlite::open("./data/db.sqlite3") will work even if "db.sqlite3" does not exist
# as long as the directory exists, it won't panic/fail!  Also, VERIFY that the path matches the `volumes:` in docker-compose.yml
RUN mkdir -p /usr/src/scsudoku/oauth_relay_service/data
COPY --from=data-copy-stage /data/* ./data/

# Copies build/.env,  build/target/release, etc
# NOTE: Files that are required for THIS service is explicitly copied, so that
#       if `cargo build --release` had failed, the missing binaries will be
#       detected and the build will fail.
COPY build/oauth_relay_service* ./
COPY build/.env ./
COPY build ./

# apparently, if .env file is UTF-16, it'll not load, it has to be UTF-8
RUN file -i .env
RUN ls -AR

# Expose ports for the web server and TCP server (see .env* files which set the env-vars)
EXPOSE ${REST_PORT}

# Copy the verification script into the container
RUN apt-get update && apt-get install -y netcat-traditional
COPY verify-kafka.sh ./
RUN chmod +x ./verify-kafka.sh

# Run the compiled binary
CMD ["source .env && ./oauth_relay_service"]

# Use the official Rust image as the base image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /usr/src/oauth_relay_service

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Copy the .env file
COPY .env .env

# Install apps dependant by 3rd party crates
RUN apt-get update && apt-get install -y libsqlite3-dev

# Build the Rust application
RUN cargo build --release

# Expose ports for the web server and TCP server
EXPOSE 8080

# Run the compiled binary
CMD ["./target/release/oauth_relay_service"]

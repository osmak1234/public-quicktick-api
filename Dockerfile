# Use the official Rust image as the base image
FROM rust:latest AS build

# Set the working directory
WORKDIR /app

# Copy only the dependency manifests to the container
COPY Cargo.toml Cargo.lock ./

# Build a dummy target to cache the dependencies
RUN mkdir src && \
  echo "fn main() {}" > src/main.rs && \
  cargo build --release && \
  rm -rf src

# Now copy the rest of the Rust project files to the container
COPY . .

# Build the Rust application
RUN cargo build --release

# Create a new image with Ubuntu as the base image
FROM ubuntu:latest

# Set the working directory in the final image
WORKDIR /app

# Copy the built binary from the previous stage, preserving the directory structure
COPY --from=build /app/target/release ./target/release

# Expose the necessary port
EXPOSE 8080

# Command to run the application
CMD ["./target/release/quicktick-api"]

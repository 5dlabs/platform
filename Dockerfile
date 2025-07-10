# Use the official Rust image as the base
FROM rust:1.75 as builder

# Create a new empty shell project for dependency caching
WORKDIR /app
RUN mkdir src
COPY Cargo.toml Cargo.toml
RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/task_board_api*

# Copy the actual code and build
COPY . .
RUN cargo build --release

# Use a slimmer runtime image
FROM debian:bookworm-slim

# Copy the built binary
COPY --from=builder /app/target/release/task-board-api /usr/local/bin/task-board-api

# Expose the port (gRPC default)
EXPOSE 50051

# Run the binary
CMD ["task-board-api"]

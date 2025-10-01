FROM rust:latest as builder
WORKDIR /app
COPY . .

# Install the GTK development libraries
RUN apt-get update && apt-get install -y libgtk-4-dev

RUN cargo check
# RUN cargo build --release

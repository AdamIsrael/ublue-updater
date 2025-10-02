# A container for building Rust/GTK applications
FROM rust:latest AS builder
WORKDIR /app
COPY . .

# Install the GTK development libraries
RUN apt-get update && apt-get install -y libgtk-4-dev libadwaita-1-dev

RUN rustup component add clippy

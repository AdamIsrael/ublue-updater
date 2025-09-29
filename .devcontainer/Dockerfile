# Use a base image with Rust pre-installed
FROM mcr.microsoft.com/devcontainers/rust:latest

# Or, if you want a more minimal base and install Rust yourself:
# FROM debian:stable-slim
# RUN apt-get update && apt-get install -y curl build-essential && \
#     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
#     echo "source $HOME/.cargo/env" >> $HOME/.bashrc

# Optional: Install additional tools
# RUN apt-get update && apt-get install -y lldb

RUN apt-get update && apt-get install -y libgtk-4-dev build-essential libvte-2.91-gtk4-dev libadwaita-1-dev

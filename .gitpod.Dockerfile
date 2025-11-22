# Dockerfile for Ona (Gitpod) workspace
# Base image with development tools pre-installed

FROM gitpod/workspace-base:latest

# Install system dependencies
USER root
RUN apt-get update && \
    apt-get install -y \
      build-essential \
      cmake \
      ninja-build \
      python3.11 \
      python3.11-venv \
      python3-pip \
      nodejs \
      npm \
      git \
      curl \
      wget \
      pkg-config \
      libssl-dev \
      libprotobuf-dev \
      protobuf-compiler \
      shellcheck \
      && rm -rf /var/lib/apt/lists/*

# Install Rust (for backend services)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

# Switch back to gitpod user
USER gitpod

# Set up Python environment
RUN python3.11 -m pip install --upgrade pip setuptools wheel

# Set working directory
WORKDIR /workspace

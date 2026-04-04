FROM mcr.microsoft.com/devcontainers/base:ubuntu

USER vscode

ENV DEBIAN_FRONTEND=noninteractive
RUN sudo apt update \
  && sudo apt upgrade --yes --no-install-recommends

# Install Rustup
RUN sudo apt update \
  && sudo apt install rustup --yes --no-install-recommends

# Install default toolchain
RUN rustup toolchain install stable

# Install additional components
RUN rustup component add clippy llvm-tools rustfmt

# Install code coverage tools
RUN cargo install cargo-llvm-cov

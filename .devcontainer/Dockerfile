FROM mcr.microsoft.com/devcontainers/base:ubuntu
ARG MACOS_SDK_VERSION=26.1

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

# Install additional targets
RUN rustup target add \
    aarch64-apple-darwin \
    aarch64-unknown-linux-musl \
    x86_64-apple-darwin \
    x86_64-unknown-linux-musl

# Install code coverage tools
RUN cargo install cargo-llvm-cov

# Download and setup MacOS SDK
RUN cd $HOME/.cargo \
  && wget https://github.com/joseluisq/macosx-sdks/releases/download/${MACOS_SDK_VERSION}/MacOSX${MACOS_SDK_VERSION}.sdk.tar.xz \
  && tar xf MacOSX${MACOS_SDK_VERSION}.sdk.tar.xz \
  && rm -rf MacOSX${MACOS_SDK_VERSION}.sdk.tar.xz

ENV SDKROOT=/home/vscode/.cargo/MacOSX${MACOS_SDK_VERSION}.sdk

COPY cargo-config.toml /home/vscode/.cargo/config.toml

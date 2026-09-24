FROM ubuntu:24.04

RUN apt-get update && apt-get install -y --no-install-recommends \
    nasm \
    gcc \
    gccgo \
    libc6-dev \
    binutils \
    gcc-mingw-w64-x86-64 \
    xorriso \
    qemu-system-x86 \
    python3 \
    python3-pytest \
    python3-pip \
    git \
    make \
    dos2unix \
    ca-certificates \
    curl \
    wget \
    && rm -rf /var/lib/apt/lists/*

# ruff backs `make lint-py` (gates test-qemu); not packaged in Ubuntu 24.04 apt.
RUN python3 -m pip install --break-system-packages "ruff==0.16.8"

# Install Rust nightly with rust-src (required for build-std)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain nightly --component rust-src
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Go 1.25 (required by TinyGo 0.40)
RUN wget -q https://go.dev/dl/go1.25.3.linux-amd64.tar.gz -O /tmp/go.tar.gz && \
    tar -C /usr/local -xzf /tmp/go.tar.gz && \
    rm /tmp/go.tar.gz
ENV PATH="/usr/local/go/bin:${PATH}"

# Install TinyGo 0.40.1
RUN wget -q https://github.com/tinygo-org/tinygo/releases/download/v0.40.1/tinygo_0.40.1_amd64.deb -O /tmp/tinygo.deb && \
    dpkg -i /tmp/tinygo.deb && \
    rm /tmp/tinygo.deb
ENV PATH="/usr/local/lib/tinygo/bin:${PATH}"

WORKDIR /src

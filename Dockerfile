FROM docker.io/library/ubuntu:24.04@sha256:b59d21599a2b151e23eea5f6602f4af4d7d31c4e236d22bf0b62b86d2e386b8f

# Stage 0: Configure APT sources cleanly and exclusively
RUN rm -f /etc/apt/sources.list /etc/apt/sources.list.d/*.list /etc/apt/sources.list.d/*.sources && \
    \
    # Add main archive sources, restricted to amd64
    echo "deb [arch=amd64] http://archive.ubuntu.com/ubuntu noble main restricted universe multiverse" > /etc/apt/sources.list.d/main_amd64.list && \
    echo "deb [arch=amd64] http://archive.ubuntu.com/ubuntu noble-updates main restricted universe multiverse" >> /etc/apt/sources.list.d/main_amd64.list && \
    echo "deb [arch=amd64] http://archive.ubuntu.com/ubuntu noble-security main restricted universe multiverse" >> /etc/apt/sources.list.d/main_amd64.list && \
    echo "deb [arch=amd64] http://archive.ubuntu.com/ubuntu noble-backports main restricted universe multiverse" >> /etc/apt/sources.list.d/main_amd64.list && \
    \
    # Add ports archive sources, restricted to non-amd64 cross-architectures
    echo "deb [arch=arm64,armhf,riscv64,ppc64el] http://ports.ubuntu.com/ubuntu-ports/ noble main restricted universe multiverse" > /etc/apt/sources.list.d/ports_cross.list && \
    echo "deb [arch=arm64,armhf,riscv64,ppc64el] http://ports.ubuntu.com/ubuntu-ports/ noble-updates main restricted universe multiverse" >> /etc/apt/sources.list.d/ports_cross.list && \
    echo "deb [arch=arm64,armhf,riscv64,ppc64el] http://ports.ubuntu.com/ubuntu-ports/ noble-security main restricted universe multiverse" >> /etc/apt/sources.list.d/ports_cross.list && \
    echo "deb [arch=arm64,armhf,riscv64,ppc64el] http://ports.ubuntu.com/ubuntu-ports/ noble-backports main restricted universe multiverse" >> /etc/apt/sources.list.d/ports_cross.list && \
    \
    # Stage 1: Install basic tools, multi-architecture support and core cross compilers
    apt-get update && \
    apt-get install -y --no-install-recommends \
    software-properties-common \
    build-essential \
    curl \
    git \
    pkg-config \
    cmake \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    gcc-arm-linux-gnueabihf \
    g++-arm-linux-gnueabihf \
    gcc-riscv64-linux-gnu \
    g++-riscv64-linux-gnu \
    gcc-powerpc64-linux-gnu \
    g++-powerpc64-linux-gnu \
    \
    # Enable multi-architecture support
    && dpkg --add-architecture arm64 \
    && dpkg --add-architecture armhf \
    && dpkg --add-architecture riscv64 \
    && dpkg --add-architecture ppc64el \
    \
    # Update package lists and install cross-architecture development libraries
    && apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl-dev \
    libssl-dev:arm64 \
    libssl-dev:armhf \
    libssl-dev:riscv64 \
    libssl-dev:ppc64el \
    zlib1g-dev \
    zlib1g-dev:arm64 \
    zlib1g-dev:armhf \
    zlib1g-dev:riscv64 \
    zlib1g-dev:ppc64el \
    \
    # Stage 2: Add PPA for GCC 14 and install specific GCC 14 versions
    && add-apt-repository ppa:ubuntu-toolchain-r/test -y \
    && apt-get update && apt-get install -y --no-install-recommends \
    gcc-14 \
    g++-14 \
    gcc-14-aarch64-linux-gnu \
    g++-14-aarch64-linux-gnu \
    gcc-14-arm-linux-gnueabihf \
    g++-14-arm-linux-gnueabihf \
    gcc-14-riscv64-linux-gnu \
    g++-14-riscv64-linux-gnu \
    gcc-14-powerpc64-linux-gnu \
    g++-14-powerpc64-linux-gnu \
    \
    # Clean up APT cache (moved to the very end of all apt operations)
    && rm -rf /var/lib/apt/lists/*

# Set working directory for subsequent commands
WORKDIR /root

# Install Rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly

# Add Rust targets (make sure PATH includes cargo for this step)
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add \
    x86_64-unknown-linux-gnu \
    i686-unknown-linux-gnu \
    aarch64-unknown-linux-gnu \
    armv7-unknown-linux-gnueabihf \
    riscv64gc-unknown-linux-gnu \
    powerpc64-unknown-linux-gnu \
    && rustup component add rust-src # Add rust source for cross-compilation features if needed

# Configure Cargo for cross-compilation linkers AND environment variables for C dependencies
# This is the key change to handle openssl-sys and similar crates gracefully.
RUN mkdir -p /root/.cargo && echo '\
[target.aarch64-unknown-linux-gnu]\n\
linker = "aarch64-linux-gnu-gcc-14"\n\
rustflags = ["-C", "link-arg=-lgcc"] # Often needed for some targets\n\
[target.aarch64-unknown-linux-gnu.env]\n\
OPENSSL_LIB_DIR = "/usr/lib/aarch64-linux-gnu"\n\
OPENSSL_INCLUDE_DIR = "/usr/include"\n\
PKG_CONFIG_PATH = "/usr/lib/aarch64-linux-gnu/pkgconfig"\n\
\n\
[target.armv7-unknown-linux-gnueabihf]\n\
linker = "arm-linux-gnueabihf-gcc-14"\n\
rustflags = ["-C", "link-arg=-lgcc"]\n\
[target.armv7-unknown-linux-gnueabihf.env]\n\
OPENSSL_LIB_DIR = "/usr/lib/arm-linux-gnueabihf"\n\
OPENSSL_INCLUDE_DIR = "/usr/include"\n\
PKG_CONFIG_PATH = "/usr/lib/arm-linux-gnueabihf/pkgconfig"\n\
\n\
[target.riscv64gc-unknown-linux-gnu]\n\
linker = "riscv64-linux-gnu-gcc-14"\n\
rustflags = ["-C", "link-arg=-lgcc"]\n\
[target.riscv64gc-unknown-linux-gnu.env]\n\
OPENSSL_LIB_DIR = "/usr/lib/riscv64-linux-gnu"\n\
OPENSSL_INCLUDE_DIR = "/usr/include"\n\
PKG_CONFIG_PATH = "/usr/lib/riscv64-linux-gnu/pkgconfig"\n\
\n\
[target.powerpc64-unknown-linux-gnu]\n\
linker = "powerpc64-linux-gnu-gcc-14"\n\
rustflags = ["-C", "link-arg=-lgcc"]\n\
[target.powerpc64-unknown-linux-gnu.env]\n\
OPENSSL_LIB_DIR = "/usr/lib/powerpc64-linux-gnu"\n\
OPENSSL_INCLUDE_DIR = "/usr/include"\n\
PKG_CONFIG_PATH = "/usr/lib/powerpc64-linux-gnu/pkgconfig"\n\
' > /root/.cargo/config.toml

# You can add a CMD here to hint users how to use it, or just leave it for an interactive shell
# CMD ["bash"]
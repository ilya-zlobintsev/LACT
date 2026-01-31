FROM rust:1.93-trixie AS builder

WORKDIR /app

RUN apt update -y && apt install -y build-essential clang libdrm-dev pkg-config ocl-icd-opencl-dev

COPY rust-toolchain.toml /app
RUN cargo --version

COPY --exclude=target* . /app

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    make build-release-headless && cp ./target/release/lact /usr/local/bin/lact

FROM debian:trixie-slim

RUN apt update -y && apt install -y libdrm-amdgpu1 libdrm-radeon1 libdrm-intel1 libdrm2 hwdata vulkan-tools && rm -rf /var/cache

COPY --from=builder /usr/local/bin/lact /usr/local/bin/lact

CMD ["lact", "daemon"]

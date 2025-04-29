FROM node:22.11.0-bookworm-slim AS webui-builder
COPY ./webui ./
RUN npm install
RUN npm run build


FROM rust:1.86-slim-bookworm AS cargo-builder
RUN mkdir -p ~/.cargo && \
    echo \
      [source.crates-io] \
      replace-with = 'rsproxy' \
      [source.rsproxy] \
      registry = "https://rsproxy.cn/crates.io-index" \
      [source.rsproxy-sparse] \
      registry = "sparse+https://rsproxy.cn/index/" \
      [registries.rsproxy] \
      index = "https://rsproxy.cn/crates.io-index" \
      [net] \
      git-fetch-with-cli = true \
    > ~/.cargo/config.toml
RUN echo \
      deb http://mirrors.aliyun.com/debian/ bookworm main non-free contrib \
      deb http://mirrors.aliyun.com/debian-security bookworm/updates main \
      deb http://mirrors.aliyun.com/debian/ bookworm-updates main non-free contrib \
    > /etc/apt/sources.list
RUN apt-get update
RUN apt-get install -y --no-install-recommends build-essential clang libssl-dev llvm pkg-config
COPY . ./
COPY --from=webui-builder /dist ./webui/dist/
RUN cargo build --release


FROM debian:bookworm-slim
ENV ADDRESS=0.0.0.0 \
    BASE= \
    LOG=info \
    PASS= \
    PORT=6868 \
    SERVE_OPTIONS= \
    WORKERS=1
RUN echo \
      deb http://mirrors.aliyun.com/debian/ bookworm main non-free contrib \
      deb http://mirrors.aliyun.com/debian-security bookworm/updates main \
      deb http://mirrors.aliyun.com/debian/ bookworm-updates main non-free contrib \
    > /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates openssl npm python3-pip && \
    pip install uv --break-system-packages
COPY --from=cargo-builder /target/release/aiter /bin/
CMD ["sh", "-c", "LOG=\"aiter=$LOG\" aiter serve --address \"$ADDRESS\" --base \"$BASE\" --port \"$PORT\" --pass \"$PASS\" --workers \"$WORKERS\" $SERVE_OPTIONS "]

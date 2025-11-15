FROM --platform=linux/amd64 docker.io/messense/rust-musl-cross:x86_64-musl AS amd64
COPY . .
RUN cargo install --profile release-with-debug --path . --root /

FROM --platform=linux/amd64 docker.io/messense/rust-musl-cross:aarch64-musl AS arm64
COPY . .
RUN cargo install --profile release-with-debug --path . --root /

FROM ${TARGETARCH} AS builder

FROM scratch
COPY --from=builder /bin/dufs /bin/dufs
STOPSIGNAL SIGINT
ENTRYPOINT ["/bin/dufs"]

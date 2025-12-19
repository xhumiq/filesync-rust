FROM docker.io/messense/rust-musl-cross:x86_64-musl AS builder
COPY . .
RUN cargo install --bin webfs --profile release-with-debug --path . --root /

FROM alpine

ENV KEYCLOAK_URL= 
ENV REALM=chat-acp
ENV CLIENT_ID=
ENV CLIENT_SECRET=
ENV BASE_PATH=/srv/media
ENV DB_PATH=/srv/data/webfs/files.db
ENV WATCH_PATH=/srv/media/Videos
ENV RSS_DAYS=7
ENV RUST_LOG=rssfeed=debug,webfs=debug
ENV RSS_OUT_PATH=/srv/aux/rss
ENV CONFIG_PATH=/etc/webfs/config.yaml

COPY --from=builder /bin/webfs /bin/webfs
STOPSIGNAL SIGINT

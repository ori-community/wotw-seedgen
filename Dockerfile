FROM rust:alpine AS build-seedgen

COPY . /app
WORKDIR /app

RUN apk --no-cache add musl-dev curl git && \
    cargo build --release


FROM alpine

WORKDIR /app

ENV RANDOMIZER_USER_DATA_DIR=/data

RUN mkdir /data && \
    adduser -H -D -u 1010 seedgen && \
    chown -R 1010 /data && \
    apk add --no-cache tini

COPY --from=build-seedgen /app/assets /app
COPY --from=build-seedgen /app/target/release/wotw-seedgen /app/wotw-seedgen

USER seedgen

ENTRYPOINT ["/sbin/tini", "--", "/app/wotw-seedgen"]

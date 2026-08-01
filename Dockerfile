FROM rust:alpine as build-seedgen

COPY . /app
WORKDIR /app

RUN apk --no-cache add musl-dev curl git && \
    cargo build --release --target-dir /app/build


FROM alpine

WORKDIR /app

ENV RANDOMIZER_USER_DATA_DIR=/data

RUN mkdir /data && \
    adduser -H -D -u 1010 seedgen && \
    chown -R 1010 /data

COPY --from=build-seedgen /app/assets /app
COPY --from=build-seedgen /app/build/release/wotw-seedgen /app/wotw-seedgen

USER seedgen

ENTRYPOINT ["/app/wotw-seedgen"]

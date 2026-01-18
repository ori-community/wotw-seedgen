FROM rust:alpine as build-seedgen

COPY . /app
WORKDIR /app

RUN apk --no-cache add musl-dev curl && \
    cargo build --release --target-dir /app/build


FROM alpine

WORKDIR /app

COPY --from=build-seedgen /app/assets /app
COPY --from=build-seedgen /app/build/release/wotw-seedgen /app/wotw-seedgen

ENTRYPOINT ["/app/wotw-seedgen"]

FROM rust:alpine as build-seedgen

COPY . /app
WORKDIR /app

RUN apk --no-cache add musl-dev && \
    cargo build --release --target-dir /app/build/output


FROM alpine

WORKDIR /app

COPY --from=build-seedgen /app/build/output/release/seedgen /app/seedgen
COPY --from=build-seedgen /app/headers /app/headers
COPY --from=build-seedgen /app/presets /app/presets
COPY --from=build-seedgen /app/areas.wotw /app/areas.wotw
COPY --from=build-seedgen /app/loc_data.csv /app/loc_data.csv
COPY --from=build-seedgen /app/state_data.csv /app/state_data.csv

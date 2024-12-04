FROM rust:latest AS builder


WORKDIR /app


COPY . .

RUN apt-get update && \
    apt-get install -y build-essential clang libssl-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*

RUN cargo build --release


FROM rust:latest AS runtime


RUN apt-get update && \
    apt-get install -y build-essential clang libssl-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/twine-aggregator /usr/local/bin/aggregator

ENTRYPOINT [ "aggregator" ]
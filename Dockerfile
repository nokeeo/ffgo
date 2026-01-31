# BUILD
From rust:latest AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/ffgo
COPY . .
RUN cargo build --release

# RUN
FROM debian:sid-slim
COPY --from=builder /usr/src/ffgo/target/release/ffgo /usr/local/bin/ffgo

CMD ["ffgo"]
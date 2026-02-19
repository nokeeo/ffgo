# BUILD
From rust:1.93-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/ffgo
COPY . .
RUN cargo build --release

# RUN
FROM debian:bookworm-slim
COPY --from=builder /usr/src/ffgo/target/release/ffgo /usr/local/bin/ffgo

RUN apt-get update && apt-get install -y debian-archive-keyring && \
    sed -i 's/Components: main/Components: main contrib non-free non-free-firmware/g' /etc/apt/sources.list.d/debian.sources

RUN apt-get update && apt-get install -y \
    ffmpeg \
    libva-drm2 \
    libva-x11-2 \
    mesa-va-drivers \
    intel-media-va-driver-non-free \
    && rm -rf /var/lib/apt/lists/*

ENV LIBVA_DRIVER_NAME=iHD

CMD ["ffgo"]

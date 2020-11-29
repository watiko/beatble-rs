ARG TARGET
ARG VERSION
ARG ARCH

FROM rustembedded/cross:${TARGET}-${VERSION}

ARG ARCH

RUN dpkg --add-architecture ${ARCH} \
 && apt-get update \
 && apt-get install -y --no-install-recommends \
    libdbus-1-dev:${ARCH} \
    libsystemd-dev:${ARCH} \
 && apt-get -y clean \
 && rm -rf /var/lib/apt/lists/*

ENV PKG_CONFIG_PATH="/usr/lib/arm-linux-gnueabihf/pkgconfig"

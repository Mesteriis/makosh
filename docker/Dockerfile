# Keep the development image aligned with rust-toolchain.toml (`stable`).
# Otherwise each newly created backend container downloads a second toolchain
# before Cargo can use the persisted dependency and target caches.
FROM rust:1.97-bookworm AS dev

ARG NODE_MAJOR=22

ENV DEBIAN_FRONTEND=noninteractive

RUN rustup component add rustfmt clippy --toolchain stable

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        bash \
        ca-certificates \
        curl \
        git \
        gnupg \
        make \
        pkg-config \
        libssl-dev \
        protobuf-compiler \
    && install -d -m 0755 /etc/apt/keyrings \
    && curl -fsSL "https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key" \
        | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg \
    && echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_${NODE_MAJOR}.x nodistro main" \
        > /etc/apt/sources.list.d/nodesource.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends nodejs \
    && corepack enable \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

CMD ["bash"]

FROM dev AS backend-builder

COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
COPY crates ./crates

FROM python:3.13-slim-bookworm AS attachment-extractor

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        poppler-utils \
        python3-pil \
        tesseract-ocr \
        tesseract-ocr-eng \
        tesseract-ocr-rus \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY docker/attachment-extractor.py /usr/local/bin/makosh-attachment-extractor

ENTRYPOINT ["/usr/bin/python3", "/usr/local/bin/makosh-attachment-extractor"]

FROM python:3.13-slim-bookworm AS attachment-extractor-gateway

COPY docker/attachment-extractor-gateway.py /usr/local/bin/makosh-attachment-extractor-gateway

ENTRYPOINT ["python3", "/usr/local/bin/makosh-attachment-extractor-gateway"]

FROM rust:alpine AS configgen_builder

WORKDIR /build

# Install dependencies
RUN apk add --no-cache \
    musl-dev

# Create a dummy main.rs file for caching
RUN mkdir src \
    && echo "fn main() {}" > src/main.rs

COPY configgen/Cargo.lock configgen/Cargo.toml . 

RUN cargo build --release

COPY configgen/src ./src

# Force cargo to detect the new main.rs file
RUN touch src/main.rs \
    && cargo build --release

FROM alpine

RUN apk add --no-cache \
    ca-certificates \
    certbot \
    nginx \
    openssl \
    s6-overlay \
    tzdata \
    yq-go

RUN ln -sf /dev/stdout /var/log/nginx/access.log \
    && ln -sf /dev/stderr /var/log/nginx/error.log

COPY --from=ghcr.io/xtls/xray-core:25.9.11 /usr/local/bin/xray /usr/bin/xray
COPY --from=configgen_builder /build/target/release/configgen /usr/bin/configgen

COPY rootfs /

ENV \
    S6_BEHAVIOUR_IF_STAGE2_FAILS=2 \
    NGINX_SSL_PATH="/etc/ssl" \
    GENERATED_CONFIG_PATH="/config/generated"

ENV \
    CERTBOT_CONFIG_PATH="${GENERATED_CONFIG_PATH}/letsencrypt" \
    XRAY_CONFIG="${GENERATED_CONFIG_PATH}/xray_config.env" \
    XRAY_RUNTIME_CONFIG="${GENERATED_CONFIG_PATH}/xray_runtime.env"

ENV \
    CERTBOT_LIVE_CERT_PATH="${CERTBOT_CONFIG_PATH}/live" \
    CERTBOT_CONFIG_PATH_WWW="${CERTBOT_CONFIG_PATH}/www"

ENTRYPOINT ["/init"]

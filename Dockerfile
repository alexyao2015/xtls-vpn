FROM alpine

RUN apk add --no-cache \
    ca-certificates \
    certbot \
    libqrencode-tools \
    nginx \
    openssl \
    python3 \
    s6-overlay \
    tzdata \
    yq-go

RUN ln -sf /dev/stdout /var/log/nginx/access.log \
    && ln -sf /dev/stderr /var/log/nginx/error.log

COPY --from=ghcr.io/xtls/xray-core:26.6.1 /usr/local/bin/xray /usr/bin/xray

COPY rootfs /

ENV \
    S6_BEHAVIOUR_IF_STAGE2_FAILS=2 \
    NGINX_SSL_PATH="/etc/ssl" \
    GENERATED_CONFIG_PATH="/config/generated"

ENV \
    CERTBOT_CONFIG_PATH="${GENERATED_CONFIG_PATH}/letsencrypt" \
    XRAY_PREVIOUS_SECRET_CONFIG="${GENERATED_CONFIG_PATH}/xray_config.env" \
    XRAY_SECRET_CONFIG="${GENERATED_CONFIG_PATH}/xray_secret_config.env" \
    XRAY_RUNTIME_CONFIG="${GENERATED_CONFIG_PATH}/xray_runtime_config.env"

ENV \
    CERTBOT_LIVE_CERT_PATH="${CERTBOT_CONFIG_PATH}/live" \
    CERTBOT_CONFIG_PATH_WWW="${CERTBOT_CONFIG_PATH}/www"

ENTRYPOINT ["/init"]

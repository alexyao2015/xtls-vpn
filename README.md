# xtls-vpn

Write a inbound.yaml file to /config/inbound.yaml to define the inbound path for the generated configs
All generated configs will be placed in /config/generated

Write a outbound.yaml file to /config/outbound.yaml to define outbound paths

## xhttp_self

Set `XHTTP_SELF_DOMAIN` to enable the `7_xhttp_self` client profile. This
domain is served by nginx with an openssl-generated self-signed certificate, and
the generated clients pin that certificate with `pinnedPeerCertSha256` / `pcs`.

The self-signed certificate is generated under the same
`${CERTBOT_LIVE_CERT_PATH}/${XHTTP_SELF_DOMAIN}` layout used by certbot, so nginx
and the client generator can share the same certificate discovery path.

Adapted from https://github.com/XTLS/Xray-core/discussions/4118

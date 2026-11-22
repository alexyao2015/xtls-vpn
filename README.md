# xtls-vpn

Write a inbound.yaml file to /config/inbound.yaml to define the inbound path for the generated configs
All generated configs will be placed in /config/generated

Write a outbound.yaml file to /config/outbound.yaml to define outbound paths

Server and REALITY SNI routing share `rootfs/etc/xray/server.yaml`.
User outbounds remain first in the generated server config.

Adapted from https://github.com/XTLS/Xray-core/discussions/4118

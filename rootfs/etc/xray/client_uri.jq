# Build VLESS share links from client_uri.yaml (fed in as JSON via `yq -o=json`).
# Replaces the former `configgen` Rust binary: yq parses the YAML, this filter
# assembles + percent-encodes the links, and qrencode renders the QR codes.
#
# Each client is a single-key map: { "<name>": { protocol, id, address, port, params } }
.clients[]
| to_entries[0] as $e          # $e.key = client name, $e.value = client body
| $e.value as $c

# Build the "k=v&k=v..." query string. jq's `tostring` already emits compact JSON
# for nested maps/arrays (e.g. the `extra` block) and the plain value for scalars,
# and `@uri` percent-encodes per RFC 3986.
| ( $c.params
    | to_entries
    | map( .key + "=" + (.value | tostring | @uri) )
    | join("&")
  ) as $query

# vless://<id>@<host>:<port>?<query>#<name>
# The id, query values and fragment are encoded; the host is left as-is (matching
# the previous configgen behaviour).
| "\($c.protocol)://\($c.id | @uri)@\($c.address):\($c.port)?\($query)#\($e.key | @uri)"

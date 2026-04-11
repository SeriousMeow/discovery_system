# Server

## Configuration (env + JSON)

This server uses a strict split of configuration sources:

- **Env variables**: listen/contact addresses, plus optional static puzzle override
- **JSON config file**: all non-address fields only

Precedence:

1. Code defaults
2. JSON overrides (non-address fields)
3. Env overrides (addresses and optional `DISCOVERY_SERVER_STATIC_PUZZLE_DIFFICULTY_BITS`)

### Static puzzle (registration)

Each request body includes a `credentials` object with at least `public_key` (base64, 32 bytes). The same
object shape is used on `/register`, `/queue/get`, and `/queue/post` so extra credential fields can be
added later. The key must satisfy: `SHA256(SHA256(public_key))` has at least `static_puzzle_difficulty_bits`
leading **zero bits**. That puzzle check runs at the HTTP API boundary (before handlers) and does not use
storage. In-memory state is only `HashMap` from public-key hex id to message queue (plus broadcast
delivery keys); handlers do not re-check the puzzle.

Default difficulty is **8** bits (suitable for local development). Raise it in production to
increase the cost of creating throwaway identities.

### Env variables (addresses and optional puzzle)

- `DISCOVERY_SERVER_LISTEN_ADDR`
  - Broadcast/P2P listen address (`SocketAddr`)
  - Default: `0.0.0.0:8080`
- `DISCOVERY_SERVER_HTTP_ADDR`
  - HTTP API bind address (`SocketAddr`)
  - Default: `0.0.0.0:8080`
- `DISCOVERY_SERVER_CONTACT_NODE`
  - Contact/bootstrap node (`SocketAddr`)
  - Optional; if omitted, the node starts without bootstrap `join`
- `DISCOVERY_SERVER_STATIC_PUZZLE_DIFFICULTY_BITS`
  - Minimum leading zero bits required on the double-SHA256 digest for `POST /register`
  - Default: `8`
  - Optional override (integer)

### JSON config file

Provide the path via:

- `DISCOVERY_SERVER_CONFIG_JSON=/path/to/server-config.json`

If the env var is set, the file **must exist** and must be valid JSON, otherwise the server fails fast on startup.

The JSON file must contain **only non-address fields**. Unknown fields are rejected.

Example:

```json
{
  "static_puzzle_difficulty_bits": 8,
  "broadcast_buffer_size": 1024,
  "new_messages_buffer_size": 1024,
  "internal_messages_buffer_size": 1024,
  "max_stored_messages": 1024,
  "membership_intervals": {
    "shuffle_passive_ms": 1000,
    "fill_active_ms": 1000,
    "sync_active_ms": 1000,
    "poll_ms": 100,
    "cleanup_ms": 1000
  },
  "broadcast_intervals": {
    "poll_ms": 100,
    "tick_ms": 100
  },
  "membership_options": {
    "max_active_view_size": 4,
    "max_passive_view_size": 24,
    "shuffle_active_view_size": 2,
    "shuffle_passive_view_size": 2,
    "active_random_walk_len": 5,
    "passive_random_walk_len": 2
  },
  "broadcast_option": {
    "ihave_timeout_ms": 500,
    "optimization_threshold": 2
  }
}
```

Notes:

- `membership_options` and `broadcast_option` may be omitted or set to `null` to keep defaults.
- `broadcast_option.ihave_timeout_ms` is in milliseconds.

### Run example

```bash
export DISCOVERY_SERVER_HTTP_ADDR="0.0.0.0:8080"
export DISCOVERY_SERVER_LISTEN_ADDR="0.0.0.0:8081"
export DISCOVERY_SERVER_CONTACT_NODE="127.0.0.1:8000"
export DISCOVERY_SERVER_CONFIG_JSON="./docker/server-config.json"

cargo run -p server
```

### Docker compose: 5-node line cluster

```bash
docker compose -f docker/docker-compose.server-line.yaml up --build
```

Quick checks:

- HTTP API on host ports `18080`, `18081`, `18082`, `18083`, `18084`
- `server1` starts without `DISCOVERY_SERVER_CONTACT_NODE`
- `server2..server5` bootstrap via static IPs `172.29.0.11..172.29.0.14` on port `8081`


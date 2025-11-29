# Twig
> Discord ↔️ Minecraft bridge bot with gRPC, OAuth, and observability baked in.

![Rust 2024 Edition](https://img.shields.io/badge/Rust-2024%20edition-orange?logo=rust)
![Discord Slash Commands](https://img.shields.io/badge/Discord-Slash%20Commands-5865F2?logo=discord&logoColor=white)
![gRPC Ready](https://img.shields.io/badge/gRPC-ready-0C7BDC)
![Database SQLite](https://img.shields.io/badge/Database-SQLite-044A64?logo=sqlite)
![License GPLv3](https://img.shields.io/badge/License-GPLv3-blue)

Twig is a Rust-powered Discord bot that keeps community management and Minecraft infrastructure in sync. It exposes Discord slash commands, a gRPC service for Minecraft proxies (Velocity, Waterfall, etc.), and a lightweight OAuth + HTTP callback layer so players can link accounts securely. The project favors reliability: SQLx migrations, Tokio-based concurrency, tracing-first logging, and optional integrations with Docker and InfluxDB for runtime insights.

Works in conjunction with the **[Twig Velocity plugin](https://github.com/caiostoduto/twig-velocity)** to provide seamless Discord-based authentication and role-based access control for Minecraft servers running behind a Velocity proxy.

## Table of contents
- [Highlights](#highlights)
- [Architecture at a glance](#architecture-at-a-glance)
- [Getting started](#getting-started)
- [Configuration reference](#configuration-reference)
- [Discord commands](#discord-commands)
- [gRPC surface](#grpc-surface)
- [HTTP + OAuth callbacks](#http--oauth-callbacks)
- [Database & migrations](#database--migrations)
- [Development workflow](#development-workflow)
- [License](#license)

## Highlights
- **Discord-native controls** powered by [Poise](https://github.com/serenity-rs/poise) to assign roles, inspect uptime, and surface health data without leaving the client.
- **Minecraft proxy bridge** implemented with [Tonic](https://github.com/hyperium/tonic) (`proto/minecraft_bridge.proto`) for proxy registration, access gating, and server event streaming.
- **Pluggable telemetry**: tracing-based logs, optional Docker socket health checks, and InfluxDB-backed uptime insights for every Minecraft node.
- **Secure onboarding** via Discord OAuth2 callbacks (`/discord/callback`) that tie Minecraft identities to Discord accounts with short-lived registration tokens.
- **Production-friendly runtime** featuring Cargo Chef multi-stage builds, Docker Compose scaffolding, and SQLx migrations that run automatically in the container entrypoint.

## Architecture at a glance
- **Discord events** feed slash command handlers in `src/commands/**` and lifecycle hooks in `src/events/**`.
- **gRPC traffic** flows through `src/grpc/**`, broadcasting events with Tokio channels so multiple proxies stay in lockstep.
- **HTTP/OAuth** endpoints in `src/http/**` finalize Discord account linking before notifying subscribers through the gRPC stream layer.
- **Persistence** lives in SQLite via SQLx with type-checked queries and migrations under `migrations/`.

## Getting started

### Prerequisites
- [Rust toolchain](https://www.rust-lang.org/tools/install) (stable, 1.80+ recommended for 2024 edition).
- `sqlx-cli` for local database tasks: `cargo install sqlx-cli --no-default-features --features native-tls,sqlite`.
- (Optional) Docker Engine if you want containerized runs or Docker socket metrics.
- (Optional) InfluxDB 2.x for uptime dashboards.

### Clone & configure
```bash
git clone https://github.com/caiostoduto/twig.git
cd twig
cp .env.example .env
# fill in DISCORD_TOKEN, owner IDs, and optional services
```

### Run locally
```bash
# Make sure migrations are applied
sqlx database create  # no-op if already exists
sqlx migrate run

# Start the bot with tracing-friendly logs
RUST_LOG=twig=debug,cargo=warn cargo run --release
```

The first launch registers slash commands globally, starts the Discord shard manager, and (optionally) spins up:
- gRPC server (`GRPC_PORT`)
- Axum HTTP server (`HTTP_PORT`) for OAuth callbacks

### Run with Docker Compose
```bash
docker compose up --build -d
# follow logs
docker compose logs -f twig
```

The container entrypoint will automatically run `sqlx db create` and `sqlx migrate run`, then launch the compiled binary. Mount `/var/run/docker.sock` to surface Docker health data via the `/status` command.

## Configuration reference

| Variable | Required | Description | Default |
| --- | --- | --- | --- |
| `DISCORD_TOKEN` | ✅ | Bot token from the Discord Developer Portal. | — |
| `DISCORD_OWNER_ID` | ✅ | Comma-separated snowflake IDs that bypass owner-only checks. | — |
| `DATABASE_URL` | ⛔️ | SQLx connection string (SQLite by default). | `sqlite:twig.sqlite` |
| `GRPC_PORT` | Optional | Port for the MinecraftBridge gRPC server. | unset (disabled) |
| `HTTP_PORT` | Optional | Axum HTTP server for redirects and `/discord/callback`. | unset (disabled) |
| `APP_URL` | Optional | Public base URL used to compute the OAuth redirect URI. | — |
| `DISCORD_OAUTH_CLIENT_ID` / `SECRET` | Optional | Needed to let players link Discord accounts through OAuth2. | — |
| `DOCKER_SOCKET` | Optional | Socket path for Docker health checks (`/var/run/docker.sock`). | unset |
| `INFLUXDB_URL`, `ORG`, `BUCKET`, `TOKEN` | Optional | Enable uptime charts for `/minecraft uptime`. | — |
| `RUST_LOG` | Optional | Tracing filter (`twig=trace,info` etc.). | `info` |

Need more knobs? See `src/utils/config.rs` for the full list and `.env.example` for common presets.

## Discord commands

| Command | Scope | Description |
| --- | --- | --- |
| `/minecraft assign` | Guild-only, owner check | Link a Discord role (or guild) to a Minecraft server record, ensuring only verified players join. |
| `/minecraft unassign` | Guild-only, owner check | Remove the role mapping for a server and release guild ownership of the proxy. |
| `/minecraft uptime` | Global | Pulls the last 6h of uptime from InfluxDB, displaying rolling windows per server. |
| `/status` | Global | One-glance view of shard counts, CPU/memory, Docker health, and uptime. |
| `/ping` | Global | Latency probe that defers the interaction and measures gateway ping. |

Command implementations live in `src/commands/**` and rely on reusable checks, embeds, and utility helpers inside `src/utils/`.

## gRPC surface

Twig exposes a single service defined in `proto/minecraft_bridge.proto`:

| RPC | Purpose |
| --- | --- |
| `RegisterProxy(ProxyRegistration)` | A proxy introduces itself (UUID + server list). Twig stores the servers and maps them to Discord guilds. |
| `CheckPlayerAccess(PlayerAccessRequest)` | Velocity plugin asks whether a player is allowed to join a target server. Twig responds with `ALLOWED`, `PROHIBITED`, or `REQUIRES_SIGNUP` plus optional auth URL + expiry. |
| `SubscribeEvents(EventSubscription)` | Server-streaming pub/sub channel that emits `ServerEvent` payloads (currently player updates, more types can follow). |

Code generation happens via `tonic-build` during `cargo build`. If you change the proto contract, rerun `cargo build` (or `cargo chef cook`) to regenerate bindings.

## HTTP + OAuth callbacks
- `GET /discord/callback` (see `src/http/discord.rs`): completes the OAuth2 dance using the `code` + `state` pair, validates short-lived registrations, links a Discord account to a Minecraft handle, and publishes a gRPC event for subscribers.
- Any other path redirects to the GitHub project page by default.

To enable OAuth:
1. Set `DISCORD_OAUTH_CLIENT_ID`, `DISCORD_OAUTH_CLIENT_SECRET`, `HTTP_PORT`, and `APP_URL`.
2. Add the redirect URI (`${APP_URL}/discord/callback`) to the Discord Developer Portal.

## Database & migrations
- SQL schema lives in `migrations/` (SQLite dialect). The first migration provisions Discord/Minecraft tables plus triggers that keep proxies and guilds tidy.
- Apply migrations with `sqlx migrate run`. SQLx validates queries at compile time; set `DATABASE_URL` before building so macros can check types.
- Containers run migrations automatically, but local developers should keep their SQLite file up to date.

## Development workflow
- `cargo fmt` and `cargo clippy --all-targets --all-features` keep the codebase tidy.
- `cargo test` currently exercises helper modules; integration coverage is planned.
- `RUST_LOG=twig=trace poise=warn` is handy while debugging interactions.
- Regenerate gRPC bindings after editing `proto/minecraft_bridge.proto` (`cargo build` handles it through `build.rs`).

Contributions are welcome—open an issue or PR describing the problem you are solving, and feel free to propose additional slash commands or event types.

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).

---

**Related projects:**
- **[Twig Velocity](https://github.com/caiostoduto/twig-velocity)** - Minecraft Velocity proxy plugin that bridges to this Discord bot via gRPC for secure player authentication and server access control
- [Velocity](https://papermc.io/software/velocity) - Modern Minecraft proxy server

**Contributing:** Issues and pull requests welcome! Please open an issue or PR describing the problem you are solving, and feel free to propose additional slash commands or event types.

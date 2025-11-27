# twig

A Discord bot built with Rust for managing Tailscale networks and monitoring Minecraft servers.

## Features

- **Tailscale Integration**: Manage Tailscale network access and roles
- **Minecraft Monitoring**: Track uptime and status of Minecraft servers
- **System Utilities**: Check bot status, latency, and system information
- **Docker Support**: Monitor Docker containers when configured

## Environment Variables

### Required
- `DISCORD_TOKEN` - Discord bot token
- `DISCORD_OWNER_ID` - Comma-separated list of Discord user IDs with owner permissions

### Optional
- `TAILSCALE_CLIENT_ID` - Tailscale OAuth client ID
- `TAILSCALE_CLIENT_SECRET` - Tailscale OAuth client secret
- `TAILSCALE_TAG` - Tailscale tag for filtering
- `DOCKER_SOCKET` - Path to Docker socket (e.g., `/var/run/docker.sock`)
- `INFLUXDB_URL` - InfluxDB instance URL
- `INFLUXDB_ORG` - InfluxDB organization
- `INFLUXDB_BUCKET` - InfluxDB bucket name
- `INFLUXDB_TOKEN` - InfluxDB authentication token
- `RUST_LOG` - Logging level (trace, debug, info, warn, error). Default: `info`

### Logging

The bot uses environment-based logging configuration via the `RUST_LOG` environment variable:

```bash
# Show all info and higher logs (default)
RUST_LOG=info cargo run

# Show debug logs
RUST_LOG=debug cargo run

# Show trace logs only for twig, warn for dependencies
RUST_LOG=twig=trace,serenity=warn cargo run

# Show only warnings and errors
RUST_LOG=warn cargo run
```

For more information on log filtering, see the [tracing-subscriber documentation](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html).

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

## Docker

A Dockerfile is provided for containerized deployment.

```bash
docker build -t twig .
docker run -d --env-file .env twig
```

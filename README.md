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

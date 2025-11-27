use std::collections::HashSet;
use std::hash::Hash;
use tonic::{Request, Response, Status};
use tracing::{info, warn};

use crate::grpc::GrpcServiceState;
use crate::grpc::minecraft_bridge::{ProxyRegistration, RegistrationResponse};

/// Register a Minecraft proxy and store its server list
pub async fn register_proxy(
    state: &GrpcServiceState,
    request: Request<ProxyRegistration>,
) -> Result<Response<RegistrationResponse>, Status> {
    let registration = request.into_inner();
    let proxy_id = registration.proxy_id.clone();
    let servers = registration.servers;

    info!(
        "[gRPC::RegisterProxy] Proxy `{}` registering with {} servers",
        &proxy_id,
        servers.len()
    );

    // Validate proxy_id
    if proxy_id.is_empty() {
        warn!("[gRPC::RegisterProxy] Received registration with empty proxy_id");
        return Err(Status::invalid_argument("proxy_id cannot be empty"));
    }

    // Check for unique server names
    if !has_unique_elements(&servers) {
        warn!(
            "[gRPC::RegisterProxy] Proxy `{}` sent duplicate server names",
            &proxy_id
        );

        return Err(Status::invalid_argument("Server names must be unique"));
    }

    // Store the registered proxy and its servers
    let _ = sqlx::query!(
        "INSERT OR IGNORE INTO minecraft_proxies (id) VALUES (?1)",
        proxy_id
    )
    .execute(&state.data.db)
    .await;

    // Build list of server names to keep
    let server_names: Vec<String> = servers.iter().map(|s| s.name.clone()).collect();

    // Delete servers for this proxy that are not in the registration list
    if !server_names.is_empty() {
        let placeholders = server_names
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "DELETE FROM minecraft_servers WHERE proxy_id = ? AND server_name NOT IN ({})",
            placeholders
        );

        let mut query_builder = sqlx::query(&query).bind(&proxy_id);
        for name in &server_names {
            query_builder = query_builder.bind(name);
        }

        let _ = query_builder.execute(&state.data.db).await;
    } else {
        // If no servers provided, delete all servers for this proxy
        let _ = sqlx::query!(
            "DELETE FROM minecraft_servers WHERE proxy_id = ?1",
            proxy_id
        )
        .execute(&state.data.db)
        .await;
    }

    let mut i = 0;
    // Insert or update servers
    for server in &servers {
        // Validate server name
        if server.name.is_empty() {
            warn!(
                "[gRPC::RegisterProxy] Proxy `{}` sent server with empty name, skipping",
                &proxy_id
            );

            continue;
        }

        // Check if server already exists
        let existing = sqlx::query!(
            "SELECT id FROM minecraft_servers WHERE proxy_id = ?1 AND server_name = ?2",
            proxy_id,
            server.name
        )
        .fetch_optional(&state.data.db)
        .await;

        // Only insert if it doesn't exist
        if existing.is_ok() && existing.unwrap().is_none() {
            let id: i64 = crate::utils::snowflake::generate_id();
            let _ = sqlx::query!(
                "INSERT INTO minecraft_servers (id, proxy_id, server_name) VALUES (?1, ?2, ?3)",
                id,
                proxy_id,
                server.name
            )
            .execute(&state.data.db)
            .await;

            i += 1;
        }
    }

    info!(
        "[gRPC::RegisterProxy] Proxy `{}` registered {} servers successfully",
        &proxy_id, i
    );

    Ok(Response::new(RegistrationResponse { success: true }))
}

/// Helper function to check if an iterator has unique elements
fn has_unique_elements<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}

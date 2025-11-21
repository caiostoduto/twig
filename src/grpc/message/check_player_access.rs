use poise::serenity_prelude::{self as serenity};
use reqwest::Url;
use std::net::Ipv4Addr;
use tonic::{Request, Response, Status};
use tracing::{info, warn};
use uuid::Uuid;

use crate::grpc::GrpcServiceState;
use crate::grpc::minecraft_bridge::{AccessStatus, PlayerAccessRequest, PlayerAccessResponse};
use crate::utils::config;
use crate::utils::minecraft::MinecraftServerType;

const DISCORD_OAUTH_BASE_URL: &str = "https://discord.com/oauth2/authorize";

/// Check if a player is allowed to join a specific server
pub async fn check_player_access(
    state: &GrpcServiceState,
    request: Request<PlayerAccessRequest>,
) -> Result<Response<PlayerAccessResponse>, Status> {
    let access_request = request.into_inner();
    let player_name = access_request.player_name;
    let player_ipv4 = access_request.player_ipv4;
    let server_name = access_request.server_name;
    let proxy_id = access_request.proxy_id;

    info!(
        "[gRPC::CheckPlayerAccess] Checking access for player `{}` ({}) to server `{}` (`{}`)",
        player_name, player_ipv4, server_name, proxy_id
    );

    // Validate player_name
    if player_name.is_empty() {
        warn!("[gRPC::CheckPlayerAccess] Player name is empty");
        return Err(Status::invalid_argument("Player name is empty"));
    }

    // Validate player_ipv4
    if player_ipv4.is_empty() || player_ipv4.parse::<Ipv4Addr>().is_err() {
        warn!("[gRPC::CheckPlayerAccess] Player IPv4 is empty or invalid");
        return Err(Status::invalid_argument("Player IPv4 is empty or invalid"));
    }

    // Validate proxy_id
    if proxy_id.is_empty() {
        warn!("[gRPC::RegisterProxy] Received registration with empty proxy_id");
        return Err(Status::invalid_argument("proxy_id is empty"));
    }

    // Validate server_name
    if server_name.is_empty() {
        warn!("[gRPC::CheckPlayerAccess] Server name is empty");
        return Err(Status::invalid_argument("Server name is empty"));
    }

    // Get minecraft_proxies.discord_guild_id
    let discord_guild_id = sqlx::query!(
        "SELECT discord_guild_id FROM minecraft_proxies WHERE id = ?1",
        proxy_id
    )
    .fetch_one(&state.data.db)
    .await
    .map_err(|e| {
        warn!(
            "[gRPC::CheckPlayerAccess] Proxy `{}` not found in minecraft_proxies: {}",
            proxy_id, e
        );

        Status::not_found("Proxy not registered")
    })?
    .discord_guild_id
    .ok_or_else(|| {
        warn!(
            "[gRPC::CheckPlayerAccess] Proxy `{}` has no discord_guild_id",
            proxy_id
        );

        Status::not_found("Proxy's guild not registered")
    })? as u64;

    // Get minecraft_servers.discord_role_id
    let (server_type, discord_role_id) = sqlx::query!(
        "SELECT server_type, discord_role_id FROM minecraft_servers WHERE proxy_id = ?1 AND server_name = ?2",
        proxy_id,
        server_name
    )
    .fetch_one(&state.data.db)
    .await
    .map_err(|e| {
        warn!(
            "[gRPC::CheckPlayerAccess] Non-lobby server `{}` (`{}`) not found in minecraft_servers: {}",
            server_name, proxy_id, e
        );

        Status::not_found("Server not registered")
    }).map(|record| (record.server_type.map(|id| id as u64), record.discord_role_id.map(|id| id as u64)))?;

    if server_type.is_none() {
        warn!(
            "[gRPC::CheckPlayerAccess] Server `{}` has no server_type configured",
            server_name
        );

        return Err(Status::not_found("Server type not configured"));
    }

    let server_type = server_type.unwrap();

    info!(
        "[gRPC::CheckPlayerAccess] Server `{}` has type `{}` and role `{:?}`",
        server_name, server_type, discord_role_id
    );

    // Check if role is configured for non-lobby servers
    if discord_role_id.is_none() && server_type != MinecraftServerType::Lobby as u64 {
        warn!(
            "[gRPC::CheckPlayerAccess] Server `{}` has no discord_role_id configured",
            server_name
        );

        return Err(Status::not_found("Server role not configured"));
    }

    // Lookup the player in the database
    // get discord_id from player_name and player_ipv4
    match sqlx::query!(
        "SELECT minecraft_users.discord_user_id 
        FROM minecraft_users 
        WHERE minecraft_users.player_name = ?1 AND minecraft_users.player_ipv4 = ?2",
        player_name,
        player_ipv4
    )
    .fetch_one(&state.data.db)
    .await
    {
        Ok(record) => match record.discord_user_id {
            None => require_registration(player_name, player_ipv4, state).await,
            Some(discord_user_id) => {
                if server_type == MinecraftServerType::Lobby as u64 {
                    check_is_guild_member(state, discord_user_id as u64, discord_guild_id).await
                } else {
                    check_user_has_role(
                        state,
                        discord_user_id as u64,
                        discord_guild_id,
                        discord_role_id.unwrap(),
                    )
                    .await
                }
            }
        },
        Err(_e) => require_registration(player_name, player_ipv4, state).await,
    }
}

async fn check_is_guild_member(
    state: &GrpcServiceState,
    discord_user_id: u64,
    discord_guild_id: u64,
) -> Result<Response<PlayerAccessResponse>, Status> {
    // Convert i64 IDs to serenity types
    let user_id = serenity::UserId::new(discord_user_id);
    let guild_id = serenity::GuildId::new(discord_guild_id);

    info!(
        "[gRPC::CheckPlayerAccess] Checking if user {} is a member of guild {}",
        user_id, guild_id
    );

    // Fetch the member from Discord API
    match state.ctx.http.get_member(guild_id, user_id).await {
        Ok(_) => {
            info!(
                "[gRPC::CheckPlayerAccess] User {} is a member of guild {} - Access granted",
                user_id, guild_id
            );

            Ok(Response::new(PlayerAccessResponse {
                status: AccessStatus::Allowed as i32,
                authentication_url: None,
                expires_in: None,
            }))
        }
        Err(e) => {
            warn!(
                "[gRPC::CheckPlayerAccess] Failed to fetch member {} in guild {}: {}",
                user_id, guild_id, e
            );

            Ok(Response::new(PlayerAccessResponse {
                status: AccessStatus::Prohibited as i32,
                authentication_url: None,
                expires_in: None,
            }))
        }
    }
}

async fn check_user_has_role(
    state: &GrpcServiceState,
    discord_user_id: u64,
    discord_guild_id: u64,
    discord_role_id: u64,
) -> Result<Response<PlayerAccessResponse>, Status> {
    // Convert i64 IDs to serenity types
    let user_id = serenity::UserId::new(discord_user_id);
    let guild_id = serenity::GuildId::new(discord_guild_id);
    let role_id = serenity::RoleId::new(discord_role_id);

    info!(
        "[gRPC::CheckPlayerAccess] Checking if user {} has role {} in guild {}",
        user_id, role_id, guild_id
    );

    // Fetch the member from Discord API
    match state.ctx.http.get_member(guild_id, user_id).await {
        Ok(member) => {
            // Check if the member has the required role
            if member.roles.contains(&role_id) {
                info!(
                    "[gRPC::CheckPlayerAccess] User {} has role {} in guild {} - Access granted",
                    user_id, role_id, guild_id
                );

                Ok(Response::new(PlayerAccessResponse {
                    status: AccessStatus::Allowed as i32,
                    authentication_url: None,
                    expires_in: None,
                }))
            } else {
                info!(
                    "[gRPC::CheckPlayerAccess] User {} does not have role {} in guild {} - Access denied",
                    user_id, role_id, guild_id
                );

                Ok(Response::new(PlayerAccessResponse {
                    status: AccessStatus::Prohibited as i32,
                    authentication_url: None,
                    expires_in: None,
                }))
            }
        }
        Err(e) => {
            warn!(
                "[gRPC::CheckPlayerAccess] Failed to fetch member {} in guild {}: {}",
                user_id, guild_id, e
            );

            Ok(Response::new(PlayerAccessResponse {
                status: AccessStatus::Prohibited as i32,
                authentication_url: None,
                expires_in: None,
            }))
        }
    }
}

async fn require_registration(
    player_name: String,
    player_ipv4: String,
    state: &GrpcServiceState,
) -> Result<Response<PlayerAccessResponse>, Status> {
    info!(
        "[gRPC::CheckPlayerAccess] Player `{}` ({}) registration required",
        player_name, player_ipv4
    );

    let mut minecraft_user_id: i64 = crate::utils::snowflake::generate_id();

    // Insert or ignore if already exists
    let _ = sqlx::query!(
        "INSERT OR IGNORE INTO minecraft_users (id, player_name, player_ipv4) VALUES (?1, ?2, ?3)",
        minecraft_user_id,
        player_name,
        player_ipv4
    )
    .execute(&state.data.db)
    .await;

    // Now fetch the id (whether it was just inserted or already existed)
    minecraft_user_id = sqlx::query!(
        "SELECT id FROM minecraft_users WHERE player_name = ?1 AND player_ipv4 = ?2",
        player_name,
        player_ipv4
    )
    .fetch_one(&state.data.db)
    .await
    .map_err(|e| {
        warn!(
            "[gRPC::CheckPlayerAccess] Failed to fetch minecraft_user id for `{}` ({}): {}",
            player_name, player_ipv4, e
        );

        Status::internal("Database error")
    })
    .unwrap()
    .id;

    let minecraft_registration_id: i64 = crate::utils::snowflake::generate_id();
    let state_token = Uuid::new_v4().to_string();

    let _ = sqlx::query!(
            "INSERT OR REPLACE INTO minecraft_registrations (id, state_token, minecraft_user_id) VALUES (?1, ?2, ?3)",
            minecraft_registration_id,
            state_token,
            minecraft_user_id
        )
        .execute(&state.data.db)
        .await.map_err(|e| {
            warn!(
                "[gRPC::CheckPlayerAccess] Failed to insert minecraft_registration for user id {}: {}",
                minecraft_user_id, e
            );

            Status::internal("Database error")
        });

    let callback_url = Url::parse(config::get_config().app_url.as_deref().unwrap())
        .unwrap()
        .join("/discord/callback")
        .unwrap();

    let mut authentication_url = Url::parse(DISCORD_OAUTH_BASE_URL).unwrap();
    authentication_url
        .query_pairs_mut()
        .append_pair(
            "client_id",
            config::get_config()
                .discord_oauth_client_id
                .as_deref()
                .unwrap(),
        )
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", callback_url.as_str())
        .append_pair("scope", "identify")
        .append_pair("state", &state_token);

    Ok(Response::new(PlayerAccessResponse {
        status: AccessStatus::RequiresSignup as i32,
        authentication_url: Some(authentication_url.to_string()),
        expires_in: Some(300),
    }))
}

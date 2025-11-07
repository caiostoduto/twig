-- SQLITE3

CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY UNIQUE NOT NULL, -- Snowflake ID
  minecraft_username TEXT UNIQUE -- Minecraft username (optional)
);

CREATE TABLE IF NOT EXISTS tailscale_machines (
  node_id TEXT PRIMARY KEY UNIQUE NOT NULL, -- Tailscale Node ID (string)
  user_id INTEGER, -- User ID (Snowflake ID)
  ipv4_address TEXT NOT NULL UNIQUE, -- Tailscale IPv4 address
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS discord_users (
  id INTEGER PRIMARY KEY UNIQUE NOT NULL, -- Discord User ID (Snowflake ID)
  user_id INTEGER UNIQUE NOT NULL, -- User ID (Snowflake ID)
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS minecraft_registrations (
  id INTEGER PRIMARY KEY UNIQUE NOT NULL, -- Registration ID (Snowflake ID)
  code INTEGER NOT NULL, -- Registration code (6-digit integer)
  minecraft_username TEXT NOT NULL, -- Minecraft username
  tailscale_machine_node_id TEXT NOT NULL -- Tailscale Machine Node ID
);

CREATE TABLE IF NOT EXISTS tailscale_tags (
  id TEXT PRIMARY KEY UNIQUE NOT NULL -- Tailscale tag (string)
);

CREATE TABLE IF NOT EXISTS discord_guilds (
  id INTEGER PRIMARY KEY UNIQUE NOT NULL, -- Discord Guild ID (Snowflake ID)
  tailscale_tag_id TEXT UNIQUE, -- Tailscale tag (string)
  FOREIGN KEY (tailscale_tag_id) REFERENCES tailscale_tags(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS minecraft_proxies (
  id TEXT PRIMARY KEY UNIQUE NOT NULL, -- Proxy ID (UUID)
  discord_guild_id INTEGER, -- Discord Guild ID (Snowflake ID)
  FOREIGN KEY (discord_guild_id) REFERENCES discord_guilds(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS minecraft_servers (
  id INTEGER PRIMARY KEY UNIQUE NOT NULL, -- Server ID (Snowflake ID)
  proxy_id TEXT NOT NULL, -- Proxy ID (UUID)
  server_name TEXT NOT NULL, -- Server name (string)
  discord_role_id INTEGER, -- Discord Role ID (Snowflake ID)
  FOREIGN KEY (proxy_id) REFERENCES minecraft_proxies(id) ON DELETE CASCADE
);

-- Trigger to set proxy's discord_guild_id to NULL when all its servers have NULL discord_role_id
CREATE TRIGGER IF NOT EXISTS nullify_proxy_guild_on_server_update
AFTER UPDATE OF discord_role_id ON minecraft_servers
WHEN NEW.discord_role_id IS NULL
BEGIN
  UPDATE minecraft_proxies
  SET discord_guild_id = NULL
  WHERE id = NEW.proxy_id
    AND NOT EXISTS (
      SELECT 1 FROM minecraft_servers
      WHERE proxy_id = NEW.proxy_id
        AND discord_role_id IS NOT NULL
    );
END;

-- Trigger to set proxy's discord_guild_id to NULL when a server is deleted
CREATE TRIGGER IF NOT EXISTS nullify_proxy_guild_on_server_delete
AFTER DELETE ON minecraft_servers
BEGIN
  UPDATE minecraft_proxies
  SET discord_guild_id = NULL
  WHERE id = OLD.proxy_id
    AND NOT EXISTS (
      SELECT 1 FROM minecraft_servers
      WHERE proxy_id = OLD.proxy_id
        AND discord_role_id IS NOT NULL
    );
END;

-- Trigger to delete discord_guild when no proxies point to it (after proxy update)
CREATE TRIGGER IF NOT EXISTS delete_orphaned_guild_on_proxy_update
AFTER UPDATE OF discord_guild_id ON minecraft_proxies
WHEN NEW.discord_guild_id IS NULL AND OLD.discord_guild_id IS NOT NULL
BEGIN
  DELETE FROM discord_guilds
  WHERE id = OLD.discord_guild_id
    AND NOT EXISTS (
      SELECT 1 FROM minecraft_proxies
      WHERE discord_guild_id = OLD.discord_guild_id
    );
END;

-- Trigger to delete discord_guild when a proxy is deleted
CREATE TRIGGER IF NOT EXISTS delete_orphaned_guild_on_proxy_delete
AFTER DELETE ON minecraft_proxies
WHEN OLD.discord_guild_id IS NOT NULL
BEGIN
  DELETE FROM discord_guilds
  WHERE id = OLD.discord_guild_id
    AND NOT EXISTS (
      SELECT 1 FROM minecraft_proxies
      WHERE discord_guild_id = OLD.discord_guild_id
    );
END;
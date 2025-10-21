-- Criado com /tailscale
CREATE TABLE IF NOT EXISTS users (
  id BIGINT PRIMARY KEY UNIQUE NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Criado com /tailscale
CREATE TABLE IF NOT EXISTS discord_users (
  id BIGINT PRIMARY KEY UNIQUE NOT NULL,
  user_id INT UNIQUE NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Criado manualmente por mim :)
CREATE TABLE IF NOT EXISTS discord_guilds (
  id BIGINT PRIMARY KEY UNIQUE NOT NULL
);

-- Criado manualmente por mim :)
CREATE TABLE IF NOT EXISTS discord_guild_roles (
  id BIGINT PRIMARY KEY UNIQUE NOT NULL,
  discord_guild_id BIGINT NOT NULL,
  tailscale_tag_id VARCHAR NOT NULL UNIQUE,
  FOREIGN KEY (tailscale_tag_id) REFERENCES tailscale_tags(id) ON DELETE CASCADE,
  FOREIGN KEY (discord_guild_id) REFERENCES discord_guilds(id) ON DELETE CASCADE
);

-- Fetch da API do Tailscale
CREATE TABLE IF NOT EXISTS tailscale_tags (
  id VARCHAR PRIMARY KEY UNIQUE NOT NULL
);

-- Criado com /tailscale via API do Tailscale
CREATE TABLE IF NOT EXISTS tailscale_authkeys (
  key_value VARCHAR PRIMARY KEY UNIQUE NOT NULL,
  revoked BOOLEAN DEFAULT FALSE,
  user_id INT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Fetch da API do Tailscale
CREATE TABLE IF NOT EXISTS tailscale_devices (
  id VARCHAR PRIMARY KEY UNIQUE NOT NULL,
  tailscale_authkey_id VARCHAR,
  user_id INT NOT NULL,
  FOREIGN KEY (tailscale_authkey_id) REFERENCES tailscale_authkeys(key_value) ON DELETE SET NULL,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Delete the guild if it has no more roles
CREATE TRIGGER IF NOT EXISTS delete_guild_when_no_roles
AFTER DELETE ON discord_guild_roles
FOR EACH ROW
BEGIN
  DELETE FROM discord_guilds
  WHERE id = OLD.discord_guild_id
  AND NOT EXISTS (
    SELECT 1
    FROM discord_guild_roles
    WHERE discord_guild_id = OLD.discord_guild_id
  );
END;

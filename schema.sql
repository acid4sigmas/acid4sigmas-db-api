
CREATE TABLE IF NOT EXISTS auth_tokens (
    jti TEXT PRIMARY KEY,
    uid BIGINT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS auth_users (
    uid BIGINT PRIMARY KEY,
    email TEXT,
    email_verified BOOLEAN DEFAULT FALSE,
    username TEXT,
    password_hash TEXT
);

CREATE TABLE IF NOT EXISTS cloudthemes (
    uid BIGINT PRIMARY KEY,
    primary_color_text TEXT,
    primary_color TEXT,
    secondary_color TEXT,
    background_color_primary TEXT,
    background_color_secondary TEXT,
    background_color_tertiary TEXT,
    primary_grey TEXT,
    secondary_grey TEXT,
    font_size TEXT,
    transparency BOOLEAN DEFAULT TRUE,
    transparency_value FLOAT NOT NULL,
    transparency_blur TEXT
);

CREATE TABLE IF NOT EXISTS cloudthemes_status (
    uid BIGINT PRIMARY KEY,
    enabled BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS users (
    uid BIGINT PRIMARY KEY,
    email TEXT,
    owner BOOLEAN DEFAULT FALSE,
    email_verified BOOLEAN DEFAULT FALSE,
    username TEXT
);

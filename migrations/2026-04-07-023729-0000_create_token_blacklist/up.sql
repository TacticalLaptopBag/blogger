-- Your SQL goes here
CREATE TABLE token_blacklist (
    jti TEXT PRIMARY KEY NOT NULL,
    expires_at TIMESTAMP NOT NULL
);
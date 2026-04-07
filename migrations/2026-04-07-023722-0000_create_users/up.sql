-- Your SQL goes here
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL UNIQUE,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT(datetime('now'))
);
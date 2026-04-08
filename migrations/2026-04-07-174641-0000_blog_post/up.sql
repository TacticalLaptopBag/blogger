-- Your SQL goes here
CREATE TABLE blog_post (
    id INTEGER PRIMARY KEY NOT NULL UNIQUE,
    title TEXT NOT NULL,
    author_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT(datetime('now')),
    modified_at TEXT DEFAULT(NULL),
    post_content TEXT NOT NULL,
    FOREIGN KEY (author_id) REFERENCES users (id)
);
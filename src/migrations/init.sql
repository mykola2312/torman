CREATE TABLE "torrent" (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    
    hash            TEXT    NOT NULL,
    name            TEXT    NOT NULL,
    destination     TEXT    NOT NULL,
    downloaded      INTEGER NOT NULL,
    uploaded        INTEGER NOT NULL,
    
    announce        TEXT    NOT NULL,
    comment         TEXT,
    created_by      TEXT,
    creation_date   INTEGER,
    publisher       TEXT,
    publisher_url   TEXT
);

CREATE UNIQUE INDEX idx_torrent_hash ON "torrent"("hash");

CREATE TABLE "file" (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,

    torrent_id      INTEGER NOT NULL,
    file_name       TEXT    NOT NULL,

    FOREIGN KEY(torrent_id)     REFERENCES "torrent"(id)
);

CREATE TABLE "category" (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,

    title           TEXT    NOT NULL,
    forum_id        INTEGER NOT NULL
);

CREATE TABLE "torrent_category" (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,

    torrent_id      INTEGER NOT NULL,
    category_id     INTEGER NOT NULL,

    FOREIGN KEY(torrent_id)     REFERENCES "torrent"(id),
    FOREIGN KEY(category_id)    REFERENCES "category"(id)
);

CREATE TABLE "deletion" (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,

    torrent_id      INTEGER NOT NULL,

    FOREIGN KEY(torrent_id)     REFERENCES "torrent"(id)
);
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
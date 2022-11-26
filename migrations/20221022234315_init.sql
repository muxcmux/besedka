CREATE TABLE sites (
  site                 VARCHAR NOT NULL UNIQUE,
  secret               BLOB NOT NULL UNIQUE DEFAULT (randomblob(48)),
  private              BOOLEAN NOT NULL DEFAULT 1,
  anonymous            BOOLEAN NOT NULL DEFAULT 0,
  moderated            BOOLEAN NOT NULL DEFAULT 1,
  comments_per_page    INTEGER NOT NULL DEFAULT 25,
  replies_per_comment  INTEGER NOT NULL DEFAULT 5,
  minutes_to_edit      INTEGER NOT NULL DEFAULT 3,
  theme                VARCHAR NOT NULL DEFAULT day_and_night
);

CREATE TRIGGER cleanup_sites_site AFTER INSERT ON sites
BEGIN
  UPDATE sites SET site = replace(replace(new.site, 'http://', ''), 'https://', '')
  WHERE rowid = new.rowid;
END;

CREATE TABLE moderators (
  name           VARCHAR NOT NULL UNIQUE,
  password       VARCHAR NOT NULL,
  avatar         TEXT,
  sid            BLOB UNIQUE
);

CREATE TABLE pages (
  id             INTEGER NOT NULL PRIMARY KEY,
  site           VARCHAR NOT NULL,
  path           VARCHAR NOT NULL,
  comments_count INTEGER NOT NULL DEFAULT 0,
  locked         BOOLEAN NOT NULL DEFAULT 0,
  UNIQUE(site, path)
);

CREATE TRIGGER cleanup_pages_site AFTER INSERT ON pages
BEGIN
  UPDATE pages SET site = replace(replace(new.site, 'http://', ''), 'https://', '')
  WHERE id = new.id;
END;

CREATE TABLE comments (
  id            INTEGER NOT NULL PRIMARY KEY,
  page_id       INTEGER NOT NULL REFERENCES pages(id) ON UPDATE CASCADE ON DELETE CASCADE,
  parent_id     INTEGER REFERENCES comments(id) ON UPDATE CASCADE ON DELETE CASCADE,
  name          VARCHAR NOT NULL DEFAULT Anonymous,
  body          VARCHAR NOT NULL,
  avatar        TEXT,
  replies_count INTEGER NOT NULL DEFAULT 0,
  reviewed      BOOLEAN NOT NULL DEFAULT 0,
  created_at    DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at    DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  token         BLOB NOT NULL DEFAULT (randomblob(48))
);

CREATE INDEX idx_comments_path      ON comments(page_id);
CREATE INDEX idx_comments_parent_id ON comments(parent_id);

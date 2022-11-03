CREATE TABLE configs (
  site                 VARCHAR NOT NULL UNIQUE,
  secret               BLOB NOT NULL UNIQUE DEFAULT (randomblob(32)),
  anonymous_comments   BOOLEAN NOT NULL DEFAULT 1,
  moderated            BOOLEAN NOT NULL DEFAULT 0,
  comments_per_page    INTEGER NOT NULL DEFAULT 25,
  replies_per_comment  INTEGER NOT NULL DEFAULT 25,
  minutes_to_edit      INTEGER NOT NULL DEFAULT 3,
  theme                VARCHAR NOT NULL DEFAULT day_and_night
);

CREATE TRIGGER cleanup_config_site AFTER INSERT ON configs
BEGIN
  UPDATE configs SET site = replace(replace(new.site, 'http://', ''), 'https://', '')
  WHERE rowid = new.rowid;
END;

INSERT INTO configs (site) VALUES('default');

CREATE TABLE users (
  id             INTEGER NOT NULL PRIMARY KEY,
  site           VARCHAR NOT NULL,
  username       VARCHAR NOT NULL DEFAULT (hex(randomblob(32))),
  name           VARCHAR NOT NULL DEFAULT Anonymous,
  password       VARCHAR,
  password_salt  VARCHAR,
  moderator      BOOLEAN NOT NULL DEFAULT 0,
  third_party_id VARCHAR,
  avatar         TEXT,
  UNIQUE(site, username)
);

CREATE TRIGGER cleanup_users_site AFTER INSERT ON users
BEGIN
  UPDATE users SET site = replace(replace(new.site, 'http://', ''), 'https://', '')
  WHERE id = new.id;
END;

CREATE INDEX idx_users_third_party_id ON users(third_party_id);

CREATE TABLE pages (
  id             INTEGER NOT NULL PRIMARY KEY,
  site           VARCHAR NOT NULL,
  path           VARCHAR NOT NULL,
  comments_count INTEGER NOT NULL DEFAULT 0,
  locked_at      DATETIME,
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
  user_id       INTEGER REFERENCES users(id) ON UPDATE CASCADE ON DELETE SET NULL,
  name          VARCHAR NOT NULL DEFAULT Anonymous,
  body          VARCHAR NOT NULL,
  avatar        TEXT,
  replies_count INTEGER NOT NULL DEFAULT 0,
  locked_at     DATETIME,
  reviewed_at   DATETIME,
  created_at    DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at    DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_comments_path      ON comments(page_id);
CREATE INDEX idx_comments_parent_id ON comments(parent_id);

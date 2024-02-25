PRAGMA journal_mode=WAL; -- persistent pragma https://sqlite.org/wal.html#persistence_of_wal_mode

CREATE TABLE kvs (
  key TEXT NOT NULL,
  value TEXT,

  PRIMARY KEY (key)
);

CREATE TABLE documents_snapshots (
  id          TEXT    NOT NULL,

  rev         TEXT    NOT NULL,

  document_type   TEXT    NOT NULL,
  updated_at      TEXT    NOT NULL,
  data            TEXT    NOT NULL,

  PRIMARY KEY (id, rev)
);

-- additional computed data
CREATE TABLE documents_refs (
  id          TEXT    NOT NULL,
  rev         TEXT    NOT NULL,

  refs        TEXT NOT NULL,

  FOREIGN KEY (id, rev) REFERENCES documents_snapshots(id, rev) ON DELETE CASCADE
);

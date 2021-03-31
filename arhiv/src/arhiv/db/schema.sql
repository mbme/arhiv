CREATE TABLE settings (
  key TEXT NOT NULL,
  value TEXT,

  PRIMARY KEY (key)
);

CREATE TABLE documents (
  id TEXT NOT NULL,
  rev NUMBER NOT NULL,
  type TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  archived BOOLEAN NOT NULL,
  refs TEXT NOT NULL,
  data TEXT NOT NULL,

  PRIMARY KEY (id)
);

CREATE TABLE documents_history (
  id TEXT NOT NULL,
  rev NUMBER NOT NULL,
  base_rev NUMBER NOT NULL,
  type TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  archived BOOLEAN NOT NULL,
  refs TEXT NOT NULL,
  data TEXT NOT NULL,

  PRIMARY KEY (id, rev)
);

-- FULL TEXT SEARCH

CREATE VIRTUAL TABLE documents_fts USING fts5(
  id UNINDEXED,
  rev UNINDEXED,
  type UNINDEXED,
  created_at UNINDEXED,
  updated_at UNINDEXED,
  archived UNINDEXED,
  refs UNINDEXED,
  data,
  content='documents',
  content_rowid='rowid'
);

CREATE TRIGGER documents_ai AFTER INSERT ON documents
    BEGIN
        INSERT INTO documents_fts (rowid, data)
        VALUES (new.rowid, new.data);
    END;

CREATE TRIGGER documents_ad AFTER DELETE ON documents
    BEGIN
        INSERT INTO documents_fts (documents_fts, rowid, data)
        VALUES ('delete', old.rowid, old.data);
    END;

CREATE TRIGGER documents_au AFTER UPDATE ON documents
    BEGIN
        INSERT INTO documents_fts (documents_fts, rowid, data)
        VALUES ('delete', old.rowid, old.data);
        INSERT INTO documents_fts (rowid, data)
        VALUES (new.rowid, new.data);
    END;

------------------------

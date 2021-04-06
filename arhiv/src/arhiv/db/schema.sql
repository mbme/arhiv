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

CREATE VIRTUAL TABLE documents_index USING fts5(
  search_data,
  content="",
  tokenize="trigram"
);

CREATE TRIGGER documents_after_insert AFTER INSERT ON documents
    BEGIN
        INSERT INTO documents_index (rowid, search_data)
        VALUES (new.rowid, extract_search_data(new.type, new.data));
    END;

CREATE TRIGGER documents_after_delete AFTER DELETE ON documents
    BEGIN
        INSERT INTO documents_index (documents_index, rowid, search_data)
        VALUES ('delete', old.rowid, extract_search_data(old.type, old.data));
    END;

CREATE TRIGGER documents_after_update AFTER UPDATE ON documents
    BEGIN
        INSERT INTO documents_index (documents_index, rowid, search_data)
        VALUES ('delete', old.rowid, extract_search_data(old.type, old.data));

        INSERT INTO documents_index (rowid, search_data)
        VALUES (new.rowid, extract_search_data(new.type, new.data));
    END;

------------------------

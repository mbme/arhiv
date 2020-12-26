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
  type TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  archived BOOLEAN NOT NULL,
  refs TEXT NOT NULL,
  data TEXT NOT NULL,

  PRIMARY KEY (id, rev)
);

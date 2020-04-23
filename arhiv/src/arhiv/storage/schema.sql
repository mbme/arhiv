BEGIN;

CREATE TABLE documents (
  id TEXT NOT NULL,
  rev NUMBER NOT NULL,

  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  archived BOOLEAN NOT NULL,

  type TEXT NOT NULL,
  refs TEXT NOT NULL,
  attachment_refs TEXT NOT NULL,
  data TEXT NOT NULL,

  PRIMARY KEY (id, rev)
);

CREATE TABLE attachments (
  id TEXT NOT NULL,
  rev NUMBER NOT NULL,

  created_at TEXT NOT NULL,
  filename TEXT NOT NULL,

  PRIMARY KEY (id)
);

COMMIT;

CREATE TABLE settings (
  key TEXT NOT NULL,
  value TEXT,

  PRIMARY KEY (key)
);

CREATE TABLE documents_snapshots (
  id          TEXT    NOT NULL,

  rev         INTEGER NOT NULL,
  prev_rev    INTEGER NOT NULL,

  snapshot_id TEXT    NOT NULL  UNIQUE,

  type        TEXT    NOT NULL,
  created_at  TEXT    NOT NULL,
  updated_at  TEXT    NOT NULL,
  archived    BOOLEAN NOT NULL,
  refs        TEXT    NOT NULL,
  data        TEXT    NOT NULL,

  PRIMARY KEY (id, rev)
);

CREATE VIEW documents AS
  SELECT a.* FROM documents_snapshots a
    INNER JOIN
        (SELECT rowid,
                ROW_NUMBER() OVER (PARTITION BY id
                                    ORDER BY CASE
                                              WHEN rev = 0 THEN 4294967295 -- u32::MAX in Rust
                                              ELSE rev
                                            END
                                    DESC) rn
        FROM documents_snapshots) b
    ON a.rowid = b.rowid WHERE b.rn = 1;

-- conflict is a
-- 1. staged document (rev = 0)
-- 2. with prev_rev != max rev of the same document
CREATE VIEW conflicts AS
  SELECT a.* FROM
          documents_snapshots a
      INNER JOIN
          (SELECT id, MAX(rev) max_rev FROM documents_snapshots GROUP BY id) b
      ON a.id = b.id
      WHERE a.rev = 0 AND a.prev_rev != b.max_rev;

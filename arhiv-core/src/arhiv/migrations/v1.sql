PRAGMA journal_mode=WAL; -- persistent pragma https://sqlite.org/wal.html#persistence_of_wal_mode

CREATE TABLE settings (
  key TEXT NOT NULL,
  value TEXT,

  PRIMARY KEY (key)
);

CREATE TABLE documents_snapshots (
  id          TEXT    NOT NULL,

  rev         INTEGER NOT NULL,
  prev_rev    INTEGER NOT NULL,

  type            TEXT    NOT NULL,
  created_at      TEXT    NOT NULL,
  updated_at      TEXT    NOT NULL,
  data            TEXT    NOT NULL,

  PRIMARY KEY (id, rev)
);

-- additional computed data
CREATE TABLE documents_refs (
  id          TEXT    NOT NULL,
  rev         INTEGER NOT NULL,

  refs        TEXT NOT NULL,

  FOREIGN KEY (id, rev) REFERENCES documents_snapshots(id, rev) ON DELETE CASCADE
);

CREATE VIEW documents_snapshots_and_refs AS
  SELECT a.rowid, a.*, b.refs
    FROM documents_snapshots a
    INNER JOIN documents_refs b
      ON a.id = b.id
      AND a.rev = b.rev;

CREATE VIEW documents AS
  SELECT a.* FROM documents_snapshots_and_refs a
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

CREATE VIEW committed_documents AS
  SELECT a.*
  FROM documents_snapshots_and_refs a
    WHERE a.rev > 0
    GROUP BY a.id HAVING MAX(a.rev);

CREATE VIEW staged_documents AS
  SELECT a.*
    FROM documents_snapshots_and_refs a
    WHERE a.rev = 0;

CREATE VIEW committed_blob_ids AS
  SELECT DISTINCT blob_refs.value AS blob_id
    FROM committed_documents AS cd, json_each(cd.refs, '$.blobs') AS blob_refs;

CREATE VIEW staged_blob_ids AS
  SELECT DISTINCT blob_refs.value AS blob_id
    FROM staged_documents AS sd, json_each(sd.refs, '$.blobs') AS blob_refs;

CREATE VIEW new_blob_ids AS
  SELECT blob_id FROM staged_blob_ids
  EXCEPT
    SELECT blob_id FROM committed_blob_ids;

CREATE VIEW used_blob_ids AS
  SELECT DISTINCT blob_refs.value AS blob_id
    FROM documents AS d, json_each(d.refs, '$.blobs') AS blob_refs;

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
  subtype         TEXT    NOT NULL,
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

CREATE VIEW documents_snapshots_and_refs AS
  SELECT a.rowid, a.*, b.refs
    FROM documents_snapshots a
    INNER JOIN documents_refs b
      ON a.id = b.id
      AND a.rev = b.rev;

CREATE VIEW documents AS
  SELECT a.* FROM documents_snapshots_and_refs a
    INNER JOIN
        (SELECT rowid, ROW_NUMBER() OVER (PARTITION BY id ORDER BY rev COLLATE REV_CMP DESC) rn
        FROM documents_snapshots) b
    ON a.rowid = b.rowid WHERE b.rn = 1;

CREATE VIEW documents_with_conflicts AS
  SELECT a.id, a.rev
  FROM documents_snapshots_and_refs a
  INNER JOIN (
      SELECT id, MAX(rev) COLLATE REV_CMP as max_rev
      FROM documents_snapshots
      WHERE rev != '{}' COLLATE REV_CMP
      GROUP BY id
  ) b ON a.id = b.id AND a.rev = b.max_rev COLLATE REV_CMP
  GROUP BY a.id
  HAVING COUNT(*) > 1;

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

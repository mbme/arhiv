use std::{panic::AssertUnwindSafe, sync::Arc, time::Instant};

use anyhow::{anyhow, bail, Context, Result};
use rusqlite::{
    functions::{Context as FunctionContext, FunctionFlags},
    types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
    Connection, Error as RusqliteError, OpenFlags,
};
use serde_json::Value;

use rs_utils::log;

use crate::{
    document_expert::DocumentExpert,
    entities::{BLOBId, DocumentData, DocumentType, Id, Revision},
    schema::DataSchema,
    KvsKey,
};

pub fn open_connection(db_file: &str, mutable: bool) -> Result<Connection> {
    let conn = Connection::open_with_flags(
        db_file,
        if mutable {
            OpenFlags::SQLITE_OPEN_READ_WRITE
        } else {
            OpenFlags::SQLITE_OPEN_READ_ONLY
        },
    )
    .context("failed to open connection")?;

    conn.pragma_update(None, "foreign_keys", true)
        .context("failed to enable foreign keys support")?;

    Ok(conn)
}

pub fn init_functions(conn: &Connection, schema: &Arc<DataSchema>) -> Result<()> {
    init_extract_refs_fn(conn, schema.clone())?;
    init_calculate_search_score_fn(conn, schema.clone())?;
    init_json_contains(conn)?;
    init_rev_cmp_collation(conn)?;
    init_views(conn)?;

    Ok(())
}

fn init_calculate_search_score_fn(conn: &Connection, schema: Arc<DataSchema>) -> Result<()> {
    // WARN: schema MUST be an Arc and MUST be moved into the closure in order for sqlite to work correctly

    let schema = AssertUnwindSafe(schema);

    let calculate_search_score = move |ctx: &FunctionContext| -> Result<usize> {
        let document_type = ctx
            .get_raw(0)
            .as_str()
            .context("document_type must be str")?;

        let document_data = ctx
            .get_raw(1)
            .as_str()
            .context("document_data must be str")?;

        let pattern = ctx.get_raw(2).as_str().context("pattern must be str")?;

        if pattern.is_empty() {
            return Ok(1);
        }

        let document_type = DocumentType::new(document_type);

        let document_data: DocumentData = serde_json::from_str(document_data)?;

        let document_expert = DocumentExpert::new(&schema);

        let result = document_expert.search(&document_type, &document_data, pattern);

        if let Err(ref err) = result {
            log::error!("calculate_search_score() failed: \n{}", err);
        }

        result
    };

    conn.create_scalar_function(
        "calculate_search_score",
        3,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 3, "called with unexpected number of arguments");

            calculate_search_score(ctx).map_err(|e| {
                RusqliteError::UserFunctionError(
                    anyhow!("calculate_search_score() failed: {e}").into(),
                )
            })
        },
    )
    .context("Failed to define function 'calculate_search_score'")
}

fn init_json_contains(conn: &Connection) -> Result<()> {
    conn.create_scalar_function(
        "json_contains",
        3,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 3, "called with unexpected number of arguments");

            let data = ctx.get_raw(0).as_str().expect("data must be str");
            let field = ctx.get_raw(1).as_str().expect("field must be str");
            let value = ctx.get_raw(2).as_str().expect("value must be str");

            json_contains(data, field, value)
                .context("json_contains() failed")
                .map_err(|e| RusqliteError::UserFunctionError(e.into()))
        },
    )
    .context("Failed to define function 'json_contains'")
}

fn init_extract_refs_fn(conn: &Connection, schema: Arc<DataSchema>) -> Result<()> {
    // WARN: schema MUST be an Arc and MUST be moved into the closure in order for sqlite to work correctly

    let schema = AssertUnwindSafe(schema);

    let extract_refs = move |ctx: &FunctionContext| -> Result<String> {
        let document_type = ctx
            .get_raw(0)
            .as_str()
            .context("document_type must be str")?;

        let document_data = ctx
            .get_raw(1)
            .as_str()
            .context("document_data must be str")?;

        let document_type = DocumentType::new(document_type);

        let document_data: DocumentData = serde_json::from_str(document_data)?;

        let document_expert = DocumentExpert::new(&schema);

        let refs = document_expert.extract_refs(&document_type, &document_data)?;

        serde_json::to_string(&refs).context("failed to serialize refs")
    };

    conn.create_scalar_function(
        "extract_refs",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 2, "called with unexpected number of arguments");

            let result = extract_refs(ctx);

            if let Err(ref err) = result {
                log::error!("extract_refs() failed: \n{:?}", err);
            }

            result.map_err(|e| RusqliteError::UserFunctionError(e.into()))
        },
    )
    .context("Failed to define function 'extract_refs'")
}

fn init_rev_cmp_collation(conn: &Connection) -> Result<()> {
    use std::cmp::Ordering;

    conn.create_collation("REV_CMP", move |rev_a, rev_b| {
        let rev_a: Option<Revision> =
            serde_json::from_str(rev_a).expect("must parse rev_a as Revision");
        let rev_b: Option<Revision> =
            serde_json::from_str(rev_b).expect("must parse rev_b as Revision");

        match (rev_a, rev_b) {
            (Some(rev_a), Some(rev_b)) => rev_a.partial_cmp(&rev_b).unwrap_or(Ordering::Equal), // conflicts are considered equal
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (None, None) => Ordering::Equal,
        }
    })
    .context("Failed to define collation 'REV_CMP'")
}

fn init_views(conn: &Connection) -> Result<()> {
    let staging = Revision::STAGED_STRING;

    conn.execute_batch(&format!("
        CREATE TEMPORARY VIEW documents_snapshots_and_refs AS
            SELECT a.rowid, a.*, b.refs
                FROM documents_snapshots a INNER JOIN documents_refs b
                ON a.id = b.id AND a.rev = b.rev;

        CREATE TEMPORARY VIEW documents AS
            SELECT a.* FROM documents_snapshots_and_refs a
                INNER JOIN
                    (SELECT rowid, ROW_NUMBER() OVER (PARTITION BY id ORDER BY rev COLLATE REV_CMP DESC) rn
                    FROM documents_snapshots) b
                ON a.rowid = b.rowid WHERE b.rn = 1;

        CREATE TEMPORARY VIEW documents_with_conflicts AS
            SELECT a.id, a.rev
            FROM documents_snapshots_and_refs a
            INNER JOIN (
                SELECT id, MAX(rev) COLLATE REV_CMP as max_rev
                FROM documents_snapshots
                WHERE rev != '{staging}'
                GROUP BY id
            ) b ON a.id = b.id AND a.rev = b.max_rev COLLATE REV_CMP
            GROUP BY a.id
            HAVING COUNT(*) > 1;

        CREATE TEMPORARY VIEW committed_documents AS
            SELECT a.*
            FROM documents_snapshots_and_refs a
                WHERE a.rev != '{staging}'
                GROUP BY a.id HAVING MAX(a.rev) COLLATE REV_CMP;

        CREATE TEMPORARY VIEW staged_documents AS
            SELECT a.*
                FROM documents_snapshots_and_refs a
                WHERE a.rev = '{staging}';

        CREATE TEMPORARY VIEW committed_blob_ids AS
            SELECT DISTINCT blob_refs.value AS blob_id
                FROM committed_documents AS cd, json_each(cd.refs, '$.blobs') AS blob_refs;

        CREATE TEMPORARY VIEW staged_blob_ids AS
            SELECT DISTINCT blob_refs.value AS blob_id
                FROM staged_documents AS sd, json_each(sd.refs, '$.blobs') AS blob_refs;

        CREATE TEMPORARY VIEW new_blob_ids AS
            SELECT blob_id FROM staged_blob_ids
            EXCEPT
                SELECT blob_id FROM committed_blob_ids;

        CREATE TEMPORARY VIEW used_blob_ids AS
            SELECT DISTINCT blob_refs.value AS blob_id
                FROM documents AS d, json_each(d.refs, '$.blobs') AS blob_refs;
      "),
    )
    .context("Failed to initialize views")?;

    Ok(())
}

fn json_contains(data: &str, field: &str, value: &str) -> Result<bool> {
    let data: Value = serde_json::from_str(data)?;

    let data = if let Some(data) = data.get(field) {
        data
    } else {
        return Ok(false);
    };

    if let Some(data) = data.as_str() {
        return Ok(data == value);
    }

    if let Some(data) = data.as_array() {
        let result = data.iter().any(|item| item.as_str() == Some(value));

        return Ok(result);
    }

    bail!("data must be string or array")
}

pub fn vacuum(db_file: &str) -> Result<()> {
    let conn = open_connection(db_file, true)?;

    let now = Instant::now();

    conn.execute("VACUUM", [])?;

    log::debug!(
        "completed VACUUM in {} seconds",
        now.elapsed().as_secs_f32()
    );

    Ok(())
}

impl FromSql for Id {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(Id::from)
    }
}

impl ToSql for Id {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self as &str))
    }
}

impl FromSql for BLOBId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()
            .map(|value| BLOBId::from_string(value).expect("must be valid BLOB id"))
    }
}

impl ToSql for BLOBId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self as &str))
    }
}

impl FromSql for DocumentType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(DocumentType::new)
    }
}

impl ToSql for DocumentType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self as &str))
    }
}

impl FromSql for KvsKey {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let (namespace, key): (String, String) =
            serde_json::from_str(value.as_str()?).map_err(|err| FromSqlError::Other(err.into()))?;

        Ok(KvsKey { namespace, key })
    }
}

impl ToSql for KvsKey {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let key = serde_json::to_string(&vec![&self.namespace, &self.key])
            .expect("failed to serialize kvs key");

        Ok(ToSqlOutput::Owned(key.into()))
    }
}

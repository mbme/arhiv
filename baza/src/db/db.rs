use std::{sync::Arc, time::Instant};

use anyhow::{bail, Context, Result};
use rusqlite::{
    functions::{Context as FunctionContext, FunctionFlags},
    types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
    Connection, Error as RusqliteError, OpenFlags,
};
use serde_json::Value;

use rs_utils::log;

use crate::{
    document_expert::DocumentExpert,
    entities::{BLOBId, DocumentClass, DocumentData, Id},
    schema::DataSchema,
    sync::Revision,
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

    Ok(())
}

fn init_calculate_search_score_fn(conn: &Connection, schema: Arc<DataSchema>) -> Result<()> {
    // WARN: schema MUST be an Arc and MUST be moved into the closure in order for sqlite to work correctly

    let calculate_search_score = move |ctx: &FunctionContext| -> Result<usize> {
        let document_type = ctx
            .get_raw(0)
            .as_str()
            .context("document_type must be str")?;

        let subtype = ctx.get_raw(1).as_str().context("subtype must be str")?;

        let document_data = ctx
            .get_raw(2)
            .as_str()
            .context("document_data must be str")?;

        let pattern = ctx.get_raw(3).as_str().context("pattern must be str")?;

        if pattern.is_empty() {
            return Ok(1);
        }

        let document_type = DocumentClass::new(document_type, subtype);

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
        4,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 4, "called with unexpected number of arguments");

            calculate_search_score(ctx)
                .context("calculate_search_score() failed")
                .map_err(|e| RusqliteError::UserFunctionError(e.into()))
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

    let extract_refs = move |ctx: &FunctionContext| -> Result<String> {
        let document_type = ctx
            .get_raw(0)
            .as_str()
            .context("document_type must be str")?;

        let subtype = ctx.get_raw(1).as_str().context("subtype must be str")?;

        let document_data = ctx
            .get_raw(2)
            .as_str()
            .context("document_data must be str")?;

        let document_type = DocumentClass::new(document_type, subtype);

        let document_data: DocumentData = serde_json::from_str(document_data)?;

        let document_expert = DocumentExpert::new(&schema);

        let refs = document_expert.extract_refs(&document_type, &document_data)?;

        serde_json::to_string(&refs).context("failed to serialize refs")
    };

    conn.create_scalar_function(
        "extract_refs",
        3,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 3, "called with unexpected number of arguments");

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
    conn.create_collation("REV_CMP", move |rev_a, rev_b| {
        let rev_a: Revision = serde_json::from_str(rev_a).expect("must parse rev_a as Revision");
        let rev_b: Revision = serde_json::from_str(rev_b).expect("must parse rev_b as Revision");

        rev_a
            .partial_cmp(&rev_b)
            .unwrap_or(std::cmp::Ordering::Equal) // conflicts are considered equal
    })
    .context("Failed to define collation 'REV_CMP'")
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
        let result = data
            .iter()
            .any(|item| item.as_str().map_or(false, |item| item == value));

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
        value.as_str().map(BLOBId::from_string)
    }
}

impl ToSql for BLOBId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self as &str))
    }
}

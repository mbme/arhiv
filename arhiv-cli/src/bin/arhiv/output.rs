use std::cmp::Ordering;

use anyhow::{Context, Result};

use baza::{
    Baza, DocumentExpert, DocumentHead, Filter, diff_document_data,
    entities::{Document, DocumentType, Id},
    schema::DataSchema,
};

use arhiv::Arhiv;

pub(crate) fn print_conflicts(arhiv: &Arhiv, json_output: bool) -> Result<()> {
    let document_expert = arhiv.baza.get_document_expert();
    let baza = arhiv.baza.open()?;
    let mut conflicts = baza.iter_conflicts().collect::<Vec<_>>();
    conflicts.sort_by_key(|head| head.get_id().to_string());

    if json_output {
        let documents = conflicts
            .iter()
            .map(|head| conflict_summary_json(&document_expert, head))
            .collect::<Result<Vec<_>>>()?;
        let total = documents.len();

        serde_json::to_writer_pretty(
            std::io::stdout(),
            &serde_json::json!({
                "documents": documents,
                "total": total,
            }),
        )?;
        return Ok(());
    }

    if conflicts.is_empty() {
        println!("No conflicts found");
        return Ok(());
    }

    println!("Conflicts: {}", conflicts.len());
    for head in conflicts {
        print_conflict_row(&document_expert, head)?;
    }

    Ok(())
}

pub(crate) fn print_document_history(arhiv: &Arhiv, id: &Id, json_output: bool) -> Result<()> {
    let document_expert = arhiv.baza.get_document_expert();
    let baza = arhiv.baza.open()?;
    get_document_head(&baza, id)?;
    let snapshots = baza.list_document_snapshots(id)?;

    if json_output {
        let snapshots = snapshots
            .iter()
            .map(|document| snapshot_json(&document_expert, document))
            .collect::<Result<Vec<_>>>()?;
        let total = snapshots.len();

        serde_json::to_writer_pretty(
            std::io::stdout(),
            &serde_json::json!({
                "id": id,
                "snapshots": snapshots,
                "total": total,
            }),
        )?;
        return Ok(());
    }

    if snapshots.is_empty() {
        println!("No committed snapshots found for document {id}");
        return Ok(());
    }

    println!("History for document {id}: {} snapshots", snapshots.len());
    for document in &snapshots {
        print_snapshot_row(&document_expert, document)?;
    }

    Ok(())
}

pub(crate) fn print_conflict_details(
    document_expert: &DocumentExpert<'_>,
    head: &DocumentHead,
    json_output: bool,
) -> Result<()> {
    let branches = sorted_original_snapshots(head);

    if json_output {
        let branches = branches
            .iter()
            .map(|document| snapshot_json(document_expert, document))
            .collect::<Result<Vec<_>>>()?;
        let staged = head
            .get_staged_document()
            .map(|document| snapshot_json(document_expert, document))
            .transpose()?;

        serde_json::to_writer_pretty(
            std::io::stdout(),
            &serde_json::json!({
                "id": head.get_id(),
                "isResolved": head.is_resolved_conflict(),
                "staged": staged,
                "branches": branches,
                "branchesCount": branches.len(),
                "snapshotsCount": head.get_snapshots_count(),
            }),
        )?;
        return Ok(());
    }

    println!("Conflict for document {}", head.get_id());
    println!("Resolved: {}", head.is_resolved_conflict());
    println!("Branches: {}", branches.len());
    println!("Snapshots: {}", head.get_snapshots_count());

    if let Some(staged) = head.get_staged_document() {
        println!();
        print_snapshot_block(document_expert, "Staged resolution", staged)?;
    }

    for (index, document) in branches.iter().enumerate() {
        println!();
        print_snapshot_block(document_expert, &format!("Branch {}", index + 1), document)?;
    }

    Ok(())
}

pub(crate) fn print_snapshot(
    document_expert: &DocumentExpert<'_>,
    document: &Document,
    json_output: bool,
) -> Result<()> {
    if json_output {
        serde_json::to_writer_pretty(
            std::io::stdout(),
            &snapshot_json(document_expert, document)?,
        )?;
    } else {
        print_snapshot_block(document_expert, "Snapshot", document)?;
    }

    Ok(())
}

pub(crate) fn print_document_data_diff(
    document_expert: &DocumentExpert<'_>,
    left_role: &str,
    left: &Document,
    right_role: &str,
    right: &Document,
) -> Result<()> {
    let left_label = document_diff_label(document_expert, left_role, left)?;
    let right_label = document_diff_label(document_expert, right_role, right)?;
    let diff = diff_document_data(&left_label, left, &right_label, right)?;

    if diff.has_changes {
        print!("{}", diff.unified_diff);
    } else {
        println!("No data differences between {left_role} and {right_role}");
    }

    Ok(())
}

fn document_diff_label(
    document_expert: &DocumentExpert<'_>,
    role: &str,
    document: &Document,
) -> Result<String> {
    let title = document_expert.get_title(&document.document_type, &document.data)?;

    Ok(format!(
        "{}: {} {} rev {} updated {} title {}",
        role,
        document.id,
        document.document_type,
        document.rev.to_safe_string(),
        document.updated_at.default_date_time_format(),
        single_line(&title)
    ))
}

pub(crate) fn print_snapshot_block(
    document_expert: &DocumentExpert<'_>,
    label: &str,
    document: &Document,
) -> Result<()> {
    let title = document_expert.get_title(&document.document_type, &document.data)?;
    let data = serde_json::to_string_pretty(&document.data)?;

    println!("{label}");
    println!("  Id: {}", document.id);
    println!("  Rev: {}", document.rev.to_safe_string());
    println!("  Type: {}", document.document_type);
    println!("  Title: {}", title);
    println!(
        "  Updated: {}",
        document.updated_at.default_date_time_format()
    );
    println!("  Data:\n{data}");

    Ok(())
}

pub(crate) fn print_snapshot_row(
    document_expert: &DocumentExpert<'_>,
    document: &Document,
) -> Result<()> {
    let title = document_expert.get_title(&document.document_type, &document.data)?;

    println!(
        "{}  {:<12}  {}  {}  {}",
        document.rev.to_safe_string(),
        document.document_type,
        document.updated_at.default_date_time_format(),
        document.id,
        single_line(&title),
    );

    Ok(())
}

fn print_conflict_row(document_expert: &DocumentExpert<'_>, head: &DocumentHead) -> Result<()> {
    let document = representative_document(head);
    let title = document_expert.get_title(&document.document_type, &document.data)?;

    println!(
        "{}  {:<12}  {}  {} [branches: {}, staged: {}]",
        document.id,
        document.document_type,
        document.updated_at.default_date_time_format(),
        single_line(&title),
        head.iter_original_snapshots().count(),
        head.is_staged(),
    );

    Ok(())
}

fn conflict_summary_json(
    document_expert: &DocumentExpert<'_>,
    head: &DocumentHead,
) -> Result<serde_json::Value> {
    let document = representative_document(head);
    let title = document_expert.get_title(&document.document_type, &document.data)?;

    Ok(serde_json::json!({
        "id": &document.id,
        "documentType": &document.document_type,
        "title": title,
        "updatedAt": document.updated_at,
        "isResolved": head.is_resolved_conflict(),
        "hasStaged": head.is_staged(),
        "branchesCount": head.iter_original_snapshots().count(),
        "snapshotsCount": head.get_snapshots_count(),
    }))
}

fn snapshot_json(
    document_expert: &DocumentExpert<'_>,
    document: &Document,
) -> Result<serde_json::Value> {
    let title = document_expert.get_title(&document.document_type, &document.data)?;

    Ok(serde_json::json!({
        "id": &document.id,
        "rev": &document.rev,
        "revSafe": document.rev.to_safe_string(),
        "documentType": &document.document_type,
        "title": title,
        "updatedAt": document.updated_at,
        "data": &document.data,
        "isStaged": document.is_staged(),
    }))
}

fn representative_document(head: &DocumentHead) -> &Document {
    head.get_staged_document()
        .unwrap_or_else(|| latest_original_snapshot(head))
}

pub(crate) fn latest_original_snapshot(head: &DocumentHead) -> &Document {
    head.iter_original_snapshots()
        .max_by(|a, b| compare_documents_by_history(a, b))
        .expect("document head must have an original snapshot")
}

pub(crate) fn sorted_original_snapshots(head: &DocumentHead) -> Vec<&Document> {
    let mut snapshots = head.iter_original_snapshots().collect::<Vec<_>>();
    snapshots.sort_by(|a, b| compare_documents_by_history(a, b));

    snapshots
}

fn compare_documents_by_history(a: &Document, b: &Document) -> Ordering {
    a.updated_at
        .cmp(&b.updated_at)
        .then_with(|| a.rev.history_cmp(&b.rev))
}

pub(crate) fn print_documents_by_ids(
    document_expert: &DocumentExpert<'_>,
    baza: &Baza,
    ids: &[Id],
    json_output: bool,
    label: &str,
    empty_message: &str,
) -> Result<()> {
    if json_output {
        let documents = ids
            .iter()
            .map(|id| {
                let head = get_document_head(baza, id)?;
                document_summary_json(document_expert, head)
            })
            .collect::<Result<Vec<_>>>()?;
        let total = documents.len();

        serde_json::to_writer_pretty(
            std::io::stdout(),
            &serde_json::json!({
                "documents": documents,
                "total": total,
            }),
        )?;
        return Ok(());
    }

    if ids.is_empty() {
        println!("{empty_message}");
        return Ok(());
    }

    println!("{label}: {}", ids.len());

    for id in ids {
        let head = get_document_head(baza, id)?;
        print_document_row(document_expert, head)?;
    }

    Ok(())
}

pub(crate) fn get_document_head<'b>(baza: &'b Baza, id: &Id) -> Result<&'b DocumentHead> {
    baza.get_document(id)
        .with_context(|| format!("Can't find document {id}"))
}

pub(crate) fn print_document_list(arhiv: &Arhiv, filter: &Filter, json_output: bool) -> Result<()> {
    let document_expert = arhiv.baza.get_document_expert();
    let baza = arhiv.baza.open()?;
    let page = baza.list_documents(filter)?;

    if json_output {
        let documents = page
            .items
            .into_iter()
            .map(|head| document_summary_json(&document_expert, head))
            .collect::<Result<Vec<_>>>()?;

        serde_json::to_writer_pretty(
            std::io::stdout(),
            &serde_json::json!({
                "documents": documents,
                "hasMore": page.has_more,
                "total": page.total,
            }),
        )?;
        return Ok(());
    }

    if page.total == 0 {
        println!("No documents found");
        return Ok(());
    }

    println!(
        "Documents: {} total, showing {}{}",
        page.total,
        page.items.len(),
        if page.has_more {
            ", more available"
        } else {
            ""
        }
    );

    for head in page.items {
        print_document_row(&document_expert, head)?;
    }

    Ok(())
}

fn document_summary_json(
    document_expert: &DocumentExpert<'_>,
    head: &DocumentHead,
) -> Result<serde_json::Value> {
    let document = representative_document(head);
    let title = document_expert.get_title(&document.document_type, &document.data)?;

    Ok(serde_json::json!({
        "id": &document.id,
        "documentType": &document.document_type,
        "title": title,
        "updatedAt": document.updated_at,
        "data": &document.data,
        "hasConflict": head.is_conflict(),
        "isStaged": head.is_staged(),
        "snapshotsCount": head.get_snapshots_count(),
    }))
}

pub(crate) fn print_document_row(
    document_expert: &DocumentExpert<'_>,
    head: &DocumentHead,
) -> Result<()> {
    let document = representative_document(head);
    let title = document_expert.get_title(&document.document_type, &document.data)?;

    println!(
        "{}  {:<12}  {}  {}{}",
        document.id,
        document.document_type,
        document.updated_at.default_date_time_format(),
        single_line(&title),
        status_flags(head)
    );

    Ok(())
}

pub(crate) fn print_document_details(
    document_expert: &DocumentExpert<'_>,
    baza: &Baza,
    head: &DocumentHead,
) -> Result<()> {
    let document = representative_document(head);
    let title = document_expert.get_title(&document.document_type, &document.data)?;
    let refs = document_expert.extract_refs(&document.document_type, &document.data)?;
    let data = serde_json::to_string_pretty(&document.data)?;

    println!("Id: {}", document.id);
    println!("Type: {}", document.document_type);
    println!("Title: {}", title);
    println!(
        "Updated: {}",
        document.updated_at.default_date_time_format()
    );
    println!("Staged: {}", head.is_staged());
    println!("Conflict: {}", head.is_conflict());
    println!("Snapshots: {}", head.get_snapshots_count());
    println!("Refs: {}", format_ids(refs.get_all_document_refs()));
    println!(
        "Backrefs: {}",
        format_ids(baza.find_document_backrefs(&document.id))
    );
    println!(
        "Collections: {}",
        format_ids(baza.find_document_collections(&document.id))
    );
    println!("Data:\n{data}");

    Ok(())
}

pub(crate) fn print_schema(
    schema: &DataSchema,
    document_type: Option<String>,
    json_output: bool,
) -> Result<()> {
    if let Some(document_type) = document_type {
        let document_type = DocumentType::new(document_type);
        let description = schema.get_data_description(&document_type)?;

        if json_output {
            serde_json::to_writer_pretty(std::io::stdout(), description)?;
            return Ok(());
        }

        println!("Document type: {}", description.document_type);
        println!("Title format: {}", description.title_format);
        println!("Fields:");
        for field in &description.fields {
            println!(
                "  {}: {:?}{}{}",
                field.name,
                field.field_type,
                if field.mandatory { ", mandatory" } else { "" },
                if field.readonly { ", readonly" } else { "" }
            );
        }

        return Ok(());
    }

    if json_output {
        serde_json::to_writer_pretty(std::io::stdout(), schema)?;
        return Ok(());
    }

    println!("Document types:");
    for document_type in schema.get_document_types() {
        println!("  {document_type}");
    }

    Ok(())
}

fn single_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn status_flags(head: &DocumentHead) -> String {
    let mut flags = Vec::new();

    if head.is_staged() {
        flags.push("staged");
    }
    if head.is_conflict() {
        flags.push("conflict");
    }

    if flags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", flags.join(", "))
    }
}

fn format_ids(ids: impl IntoIterator<Item = Id>) -> String {
    let mut ids = ids.into_iter().map(|id| id.to_string()).collect::<Vec<_>>();
    ids.sort();

    if ids.is_empty() {
        "-".to_string()
    } else {
        ids.join(", ")
    }
}

pub(crate) fn print_document(document: &Document) {
    println!("[{} {}]", document.document_type, document.id);
}

use std::{collections::HashSet, sync::Mutex};

use anyhow::*;
use arhiv_core::schema::SCHEMA;
use lazy_static::*;
use rust_embed::RustEmbed;
use serde::Serialize;
use serde_json::{json, Value};
use tera::{Context as TeraContext, Tera};

use crate::utils::get_file_hash;

lazy_static! {
    pub static ref TEMPLATES: Templates =
        Templates::new(json!({ //
            "nav_document_types": get_nav_document_types(),
        }))
        .expect("failed to init templates");
}

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/templates"]
struct TemplateAssets;

pub struct Templates {
    global_context: TeraContext,

    #[cfg(debug_assertions)]
    data: Mutex<(Tera, HashSet<u64>)>,

    #[cfg(not(debug_assertions))]
    tera: Tera,
}

impl Templates {
    pub fn new(global_context: Value) -> Result<Self> {
        let mut tera = Tera::default();
        tera.autoescape_on(vec![".html.tera"]);

        let global_context = TeraContext::from_value(json!({ "global": global_context }))?;

        Templates::new_impl(tera, global_context)
    }

    #[cfg(debug_assertions)]
    fn new_impl(mut tera: Tera, global_context: TeraContext) -> Result<Self> {
        let templates = TemplateAssets::list_template_files();
        let hashes = get_temlate_files_hashes(&templates);

        tera.add_raw_templates(templates)?;

        let data = Mutex::new((tera, hashes));

        Ok(Templates {
            data,
            global_context,
        })
    }

    #[cfg(not(debug_assertions))]
    fn new_impl(mut tera: Tera, global_context: TeraContext) -> Result<Self> {
        let templates = list_template_files();
        tera.add_raw_templates(templates)?;

        Ok(Templates {
            tera,
            global_context,
        })
    }

    #[cfg(debug_assertions)]
    pub fn render(&self, template_name: &str, context: impl Serialize) -> Result<String> {
        let mut context = TeraContext::from_value(serde_json::to_value(context)?)?;
        context.extend(self.global_context.clone());

        let templates = TemplateAssets::list_template_files();
        let hashes = get_temlate_files_hashes(&templates);

        let mut data = self.data.lock().unwrap();
        // reload templates if hashes changed
        if data.1 != hashes {
            data.0.add_raw_templates(templates)?;
            data.1 = hashes;
        }

        return data
            .0
            .render(template_name, &context)
            .context(anyhow!("failed to render template '{}'", template_name));
    }

    #[cfg(not(debug_assertions))]
    pub fn render(&self, template_name: &str, context: Value) -> Result<String> {
        let mut context = TeraContext::from_value(context)?;
        context.extend(self.global_context.clone());

        return self
            .tera
            .render(template_name, &context)
            .context(anyhow!("failed to render template '{}'", template_name));
    }
}

impl TemplateAssets {
    fn get_template_data(name: impl AsRef<str>) -> Result<String> {
        let name = name.as_ref();

        let data = TemplateAssets::get(name).context(anyhow!("can't find template '{}'", name))?;

        String::from_utf8(data.into()).context(anyhow!("template '{}' isn't valid utf8", name))
    }

    fn list_template_files() -> Vec<TemplateFile> {
        TemplateAssets::iter()
            .map(|file_name| {
                let file_name: String = file_name.into();
                let data =
                    TemplateAssets::get_template_data(&file_name).expect("template must exist");

                (file_name, data)
            })
            .collect()
    }
}

type TemplateFile = (String, String);

fn get_temlate_files_hashes(files: &Vec<TemplateFile>) -> HashSet<u64> {
    files
        .iter()
        .map(|file| get_file_hash(&file.0, &file.1))
        .collect::<HashSet<_>>()
}

const IGNORED_DOCUMENT_TYPES: &[&'static str] = &["tombstone", "attachment", "task"];

fn get_nav_document_types() -> Vec<&'static str> {
    SCHEMA
        .modules
        .iter()
        .map(|module| module.document_type)
        .filter(|document_type| !IGNORED_DOCUMENT_TYPES.contains(document_type))
        .collect()
}

#[cfg(debug_assertions)]
use std::collections::HashSet;

use anyhow::*;
use lazy_static::*;
use rust_embed::RustEmbed;
use serde::Serialize;
use tera::{Context as TeraContext, Tera};

lazy_static! {
    pub static ref TEMPLATES: Templates = Templates::new().expect("failed to init templates");
}

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/templates"]
struct TemplateAssets;

pub struct Templates {
    #[cfg(debug_assertions)]
    data: std::sync::Mutex<(Tera, HashSet<u64>)>,

    #[cfg(not(debug_assertions))]
    tera: Tera,
}

impl Templates {
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();
        tera.autoescape_on(vec![".html.tera"]);

        Templates::new_impl(tera)
    }

    #[cfg(debug_assertions)]
    fn new_impl(mut tera: Tera) -> Result<Self> {
        let templates = TemplateAssets::list_template_files();
        let hashes = get_temlate_files_hashes(&templates);

        tera.add_raw_templates(templates)?;

        let data = std::sync::Mutex::new((tera, hashes));

        Ok(Templates { data })
    }

    #[cfg(not(debug_assertions))]
    fn new_impl(mut tera: Tera) -> Result<Self> {
        let templates = TemplateAssets::list_template_files();
        tera.add_raw_templates(templates)?;

        Ok(Templates { tera })
    }

    #[cfg(debug_assertions)]
    pub fn render(&self, template_name: &str, context: impl Serialize) -> Result<String> {
        let context = TeraContext::from_value(serde_json::to_value(context)?)?;

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
    pub fn render(&self, template_name: &str, context: impl Serialize) -> Result<String> {
        let context = TeraContext::from_value(serde_json::to_value(context)?)?;

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

#[cfg(debug_assertions)]
fn get_temlate_files_hashes(files: &Vec<TemplateFile>) -> HashSet<u64> {
    use crate::utils::get_file_hash;

    files
        .iter()
        .map(|file| get_file_hash(&file.0, &file.1))
        .collect::<HashSet<_>>()
}

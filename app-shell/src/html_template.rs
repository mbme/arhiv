use crate::AppShellBuilder;
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;

pub enum AppSource {
    JSFile(String),
    JSSource(String),
    HTMLFile(String),
    HTMLSource(String),
}

impl AppSource {
    pub fn get_base_path(&self) -> String {
        match &self {
            AppSource::JSFile(path) => format!(
                "file:{}",
                Path::new(path).parent().unwrap().to_string_lossy()
            ),
            AppSource::HTMLFile(path) => format!(
                "file:{}",
                Path::new(path).parent().unwrap().to_string_lossy()
            ),
            _ => format!(
                "file:{}",
                env::current_dir()
                    .expect("failed to get current dir")
                    .to_str()
                    .expect("path must be a string")
            ),
        }
    }

    pub fn render(&self, builder: &AppShellBuilder) -> String {
        let script = match self {
            AppSource::JSFile(path) => format!("<script src=\"file:{}\" defer></script>", path),

            // wrap JS code into commented out HTML comments to prevent '</script>' string break the app
            // https://stackoverflow.com/a/28643409
            AppSource::JSSource(source) => format!("<script>/*<!--*/{}/*-->*/</script>", source),

            AppSource::HTMLFile(path) => fs::read_to_string(path).expect("HTML file must exist"),
            AppSource::HTMLSource(source) => source.clone(),
        };

        format!(
            r#"
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">

    <title>{}</title>

    <script>
      // RPC URL
      {}

      // RPC CLIENT
      {}

      // JS VARIABLES
      window.JS_VARIABLES = {};
    </script>
  </head>

  <body>
    <div id="root"></div>

    {}
  </body>
</html>
"#,
            builder.title,
            // RPC URL
            if builder.server_mode {
                format!("window.RPC_URL = '{}';", builder.get_rpc_url())
            } else {
                "".to_string()
            },
            // RPC CLIENT
            if builder.server_mode {
                include_str!("./rpc.network.js")
            } else {
                include_str!("./rpc.js")
            },
            // JS VARIABLES
            serde_json::to_string_pretty(&builder.js_variables)
                .expect("must be able to serialize JS variables"),
            script
        )
    }
}

impl fmt::Display for AppSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", {
            match self {
                AppSource::HTMLFile(path) => format!("HTML file {}", path),
                AppSource::HTMLSource(_) => "HTML string".to_owned(),
                AppSource::JSFile(path) => format!("JS file {}", path),
                AppSource::JSSource(_) => "JS string".to_owned(),
            }
        })
    }
}

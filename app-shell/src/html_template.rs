use crate::AppShellBuilder;
use std::fs;

pub enum AppSource {
    JSFile(String),
    JSSource(String),
    HTMLFile(String),
    HTMLSource(String),
}

impl AppSource {
    pub fn get_base_path(&self) -> Option<String> {
        match &self {
            AppSource::JSFile(path) => Some(format!("file://{}", path)),
            AppSource::HTMLFile(path) => Some(format!("file://{}", path)),
            _ => None,
        }
    }

    pub fn render(&self, builder: &AppShellBuilder) -> String {
        let script = match self {
            AppSource::JSFile(path) => format!("<script src=\"file://{}\" defer></script>", path),
            AppSource::JSSource(source) => format!("<script>{}</script>", source),

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
      {}
      {}
    </script>
  </head>

  <body>
    <div id="root"></div>

    {}
  </body>
</html>
"#,
            builder.title,
            if builder.server_mode {
                format!("window.RPC_URL = '{}';", builder.get_rpc_url())
            } else {
                "".to_string()
            },
            if builder.server_mode {
                include_str!("./rpc.network.js")
            } else {
                include_str!("./rpc.js")
            },
            script
        )
    }
}

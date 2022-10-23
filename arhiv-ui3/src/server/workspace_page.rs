use anyhow::{Context, Result};

use super::app::{App, AppResponse};

impl App {
    pub fn workspace_page(&self) -> Result<AppResponse> {
        let schema =
            serde_json::to_string(self.arhiv.get_schema()).context("failed to serialize schema")?;

        let content = format!(
            r#"
            <!DOCTYPE html>
            <html lang="en" dir="ltr">
                <head>
                    <title>Workspace</title>

                    <meta charset="UTF-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />

                    <link rel="icon" type="image/svg+xml" href="/public/favicon.svg" />
                    <link rel="stylesheet" href="/public/workspace.css" />
                </head>
                <body>
                    <main></main>

                    <div id="modal-root"></div>

                    <script>
                        window.SCHEMA = {schema};
                    </script>

                    <script src="/public/workspace.js"></script>
                </body>
            </html>"#
        );

        Ok(AppResponse::fragment(content))
    }
}

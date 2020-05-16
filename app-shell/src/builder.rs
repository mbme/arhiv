use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

pub struct AppShellBuilder {
    pub(crate) app_id: String,
    pub(crate) title: String,
    pub(crate) default_size: (i32, i32),
    pub(crate) show_inspector: bool,
    pub(crate) data_dir: Option<String>,
    pub(crate) actions: HashMap<String, Box<dyn Fn(Value) -> Value>>,
}

impl AppShellBuilder {
    pub fn create<S: Into<String>>(app_id: S) -> Self {
        let app_id = app_id.into();

        assert_eq!(gio::Application::id_is_valid(&app_id), true);

        AppShellBuilder {
            app_id,
            title: "".to_string(),
            default_size: (800, 600),
            data_dir: None,
            show_inspector: false,
            actions: HashMap::new(),
        }
    }

    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_default_size(mut self, width: i32, height: i32) -> Self {
        self.default_size = (width, height);
        self
    }

    pub fn show_inspector(mut self) -> Self {
        self.show_inspector = true;
        self
    }

    pub fn with_data_dir<S: Into<String>>(mut self, data_dir: S) -> Self {
        let data_dir = data_dir.into();

        let path = Path::new(&data_dir);

        if !path.is_absolute() {
            panic!("data_dir must be absolute, got {}", data_dir);
        }

        self.data_dir = Some(data_dir);
        self
    }

    pub fn with_action<S, F>(mut self, action: S, handler: F) -> Self
    where
        S: Into<String>,
        F: Fn(Value) -> Value + 'static,
    {
        self.actions.insert(action.into(), Box::new(handler));
        self
    }
}

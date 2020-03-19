use serde_json::Value;
use std::rc::Rc;

pub struct AppShellBuilder {
    pub(crate) app_id: String,
    pub(crate) title: String,
    pub(crate) default_size: (i32, i32),
    pub(crate) show_inspector: bool,
    pub(crate) action_handler: Option<Rc<dyn Fn(String, Value) -> Value>>,
}

impl AppShellBuilder {
    pub fn create(app_id: String) -> AppShellBuilder {
        assert_eq!(gio::Application::id_is_valid(&app_id), true);

        AppShellBuilder {
            app_id,
            title: "".to_string(),
            default_size: (800, 600),
            show_inspector: false,
            action_handler: None,
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
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

    pub fn with_rpc(mut self, action_handler: Rc<dyn Fn(String, Value) -> Value>) -> Self {
        self.action_handler = Some(action_handler);
        self
    }
}

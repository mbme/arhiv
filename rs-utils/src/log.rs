use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use log::{debug, error, info, trace, warn, Level};

fn setup_logger_with_level(log_level: u64) {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{},hyper=info,mdns_sd=info", get_level_filter(log_level)).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub fn setup_error_logger() {
    setup_logger_with_level(0);
}

pub fn setup_warn_logger() {
    setup_logger_with_level(1);
}

pub fn setup_logger() {
    setup_logger_with_level(2);
}

pub fn setup_debug_logger() {
    setup_logger_with_level(3);
}

pub fn setup_trace_logger() {
    setup_logger_with_level(4);
}

fn get_level_filter(log_level: u64) -> log::LevelFilter {
    match log_level {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    }
}

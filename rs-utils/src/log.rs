use log::LevelFilter;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan, time::ChronoLocal},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

pub use log::{debug, error, info, trace, warn, Level};

const DEFAULT_LOG_LEVELS: &str =
    "hyper=info,h2=info,rustls=info,mdns_sd=info,axum::rejection=trace";

#[cfg(target_os = "android")]
pub fn setup_android_logger(package: &str) {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(format!(
            "debug,{DEFAULT_LOG_LEVELS}"
        )))
        .with(tracing_android::layer(package).expect("failed to build android tracing subscriber"))
        .init();
}

#[cfg(not(target_os = "android"))]
pub fn setup_android_logger(_package: &str) {
    unreachable!()
}

fn setup_logger_with_level(log_level: LevelFilter) {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{log_level},{DEFAULT_LOG_LEVELS}").into()),
        )
        .with(
            fmt::Layer::new()
                .compact()
                .with_timer(ChronoLocal::new("[%Y-%m-%d][%H:%M:%S]".to_string()))
                .with_span_events(FmtSpan::CLOSE),
        )
        .init();
}

pub fn setup_error_logger() {
    setup_logger_with_level(LevelFilter::Error);
}

pub fn setup_warn_logger() {
    setup_logger_with_level(LevelFilter::Warn);
}

pub fn setup_logger() {
    setup_logger_with_level(LevelFilter::Info);
}

pub fn setup_debug_logger() {
    setup_logger_with_level(LevelFilter::Debug);
}

pub fn setup_trace_logger() {
    setup_logger_with_level(LevelFilter::Trace);
}

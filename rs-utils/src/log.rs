use std::panic;

use time::{UtcOffset, macros::format_description};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan, time::OffsetTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

pub use tracing::{Level, debug, error, info, trace, warn};

const DEFAULT_LOG_LEVELS: &str =
    "hyper=info,h2=info,rustls=info,axum::rejection=trace,i18n_embed=warn,keyring=info";

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
    let offset = UtcOffset::current_local_offset().expect("should get local offset!");
    let timer = OffsetTime::new(
        offset,
        format_description!(
            "[month repr:short] [day padding:zero] [year] [hour padding:zero]:[minute padding:zero]:[second padding:zero]"
        ),
    );

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{log_level},{DEFAULT_LOG_LEVELS}").into()),
        )
        .with(
            fmt::Layer::new()
                .compact()
                .with_timer(timer)
                .with_span_events(FmtSpan::CLOSE)
                .with_writer(std::io::stderr),
        )
        .init();
}

pub fn setup_error_logger() {
    setup_logger_with_level(LevelFilter::ERROR);
}

pub fn setup_warn_logger() {
    setup_logger_with_level(LevelFilter::WARN);
}

pub fn setup_logger() {
    setup_logger_with_level(LevelFilter::INFO);
}

pub fn setup_debug_logger() {
    setup_logger_with_level(LevelFilter::DEBUG);
}

pub fn setup_trace_logger() {
    setup_logger_with_level(LevelFilter::TRACE);
}

pub fn setup_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        if let Some(location) = panic_info.location() {
            error!(
                "Panic occurred: {} at {}:{}",
                panic_info
                    .payload()
                    .downcast_ref::<&str>()
                    .unwrap_or(&"Unknown"),
                location.file(),
                location.line(),
            );
        } else {
            error!("Panic occurred: {}", panic_info);
        }
    }));
}

use log::LevelFilter;
use time::{macros::format_description, util::local_offset, UtcOffset};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan, time::OffsetTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

pub use log::{debug, error, info, trace, warn, Level};

fn setup_logger_with_level(log_level: LevelFilter) {
    unsafe {
        // otherwise local offset cannot be determined in multithreaded environment (i.e. inside tokio)
        local_offset::set_soundness(local_offset::Soundness::Unsound);
    }
    let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    let timer = OffsetTime::new(offset, format_description!("[hour]:[minute]:[second]"));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{},hyper=info,mdns_sd=info", log_level).into()),
        )
        .with(
            fmt::Layer::new()
                .compact()
                .with_timer(timer)
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

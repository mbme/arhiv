use std::path::PathBuf;

pub use log::{debug, error, info, trace, warn, Level};

fn setup_logger_with_level(log_level: u64) {
    // ignore error when global logger has already been configured
    let _ = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ));
        })
        .level(get_level_filter(log_level))
        .level_for("hyper", log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply();
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

pub fn setup_file_logger(log_file: PathBuf) {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ));
        })
        .level(log::LevelFilter::Debug)
        .level_for("hyper", log::LevelFilter::Info)
        .chain(fern::log_file(log_file).expect("can't access log file"))
        .chain(std::io::stdout())
        .apply()
        .expect("setting default logger failed");
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

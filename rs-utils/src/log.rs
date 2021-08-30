use std::path::PathBuf;

pub use log::{debug, error, info, trace, warn, Level};

pub fn setup_logger_with_level(log_level: u64) {
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
        .level(get_level_filter(log_level))
        .level_for("hyper", log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
        .expect("setting default logger failed");
}

pub fn setup_logger() {
    setup_logger_with_level(0);
}

pub fn setup_debug_logger() {
    setup_logger_with_level(1);
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
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    }
}

mod cli;
mod commands;
mod output;
mod server;
mod session;

use clap::Parser;

use arhiv::Arhiv;
use baza_common::{init_global_rayon_threadpool, log};

use crate::{cli::CLIArgs, commands::handle_command};

fn main() {
    let args = CLIArgs::parse();

    match args.verbose {
        0 => log::setup_warn_logger(),
        1 => log::setup_logger(),
        2 => log::setup_debug_logger(),
        _ => log::setup_trace_logger(),
    };

    let worker_threads_count = Arhiv::optimal_number_of_worker_threads();
    log::debug!("Using {worker_threads_count} worker threads");

    init_global_rayon_threadpool(worker_threads_count)
        .expect("Failed to init global rayon thread pool");

    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.worker_threads(worker_threads_count);
    builder.enable_all();
    let runtime = builder.build().expect("Failed to create tokio runtime");

    runtime
        .block_on(handle_command(args.command))
        .expect("Failed to handle command");
}

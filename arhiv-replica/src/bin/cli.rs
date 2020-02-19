use arhiv_replica::replica::Replica;
use clap::{crate_version, App, Arg};
use path_absolutize::*;
use std::path::Path;

fn main() {
    let mut app = App::new("arhiv-replica")
        .subcommand(
            App::new("init")
                .about("Initialize replica on local machine")
                .arg(
                    Arg::with_name("root-dir")
                        .help("Replica root directory")
                        .required(true),
                )
                .arg(
                    Arg::with_name("primary-url")
                        .help("Primary arhiv server address")
                        .required(true),
                ),
        )
        .subcommand(App::new("sync").about("Trigger sync with primary server"))
        .version(crate_version!());

    let matches = app.clone().get_matches();

    match matches.subcommand() {
        ("init", Some(matches)) => {
            let root_dir = Path::new(matches.value_of("root-dir").unwrap());
            let primary_url = matches.value_of("primary-url").unwrap();

            Replica::create(
                root_dir.absolutize().unwrap().to_str().unwrap(),
                primary_url,
            )
            .expect("must be able to create replica");
        }
        _ => app.print_help().unwrap(),
    }
}

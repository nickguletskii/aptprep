use clap::Arg;
use tracing::Level;
use tracing_subscriber;

#[derive(Debug, Clone)]
pub enum Command {
    Lock {
        config_path: String,
        lockfile_path: String,
    },
    Download {
        config_path: String,
        lockfile_path: String,
    },
}

pub struct Args {
    pub command: Command,
    pub log_level: Level,
}

pub fn parse_args() -> Args {
    let matches = clap::Command::new("aptprep")
        .version("1.0.0")
        .author("Nick Guletskii")
        .about("Resolve all Debian package dependencies needed to install a given set of Debian packages behind an air gap")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Sets the level of verbosity")
                .action(clap::ArgAction::Count)
                .global(true),
        )
        .subcommand(
            clap::Command::new("lock")
                .about("Download package lists, resolve dependencies and create lockfile")
                .arg(
                    Arg::new("config")
                        .short('c')
                        .long("config")
                        .value_name("FILE")
                        .help("Sets a custom config file")
                        .required(false)
                        .default_value("config.yaml"),
                )
                .arg(
                    Arg::new("lockfile")
                        .short('l')
                        .long("lockfile")
                        .value_name("FILE")
                        .help("Sets the output lockfile path")
                        .required(false)
                        .default_value("aptprep.lock"),
                ),
        )
        .subcommand(
            clap::Command::new("download")
                .about("Read lockfile and download all required packages")
                .arg(
                    Arg::new("config")
                        .short('c')
                        .long("config")
                        .value_name("FILE")
                        .help("Sets a custom config file")
                        .required(false)
                        .default_value("config.yaml"),
                )
                .arg(
                    Arg::new("lockfile")
                        .short('l')
                        .long("lockfile")
                        .value_name("FILE")
                        .help("Sets the input lockfile path")
                        .required(false)
                        .default_value("aptprep.lock"),
                ),
        )
        .get_matches();

    let log_level = match matches.get_count("verbose") {
        0 => Level::INFO,
        1 => Level::DEBUG,
        _ => Level::TRACE,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(log_level.into())
                .from_env_lossy()
                .add_directive("pubgrub=warn".parse().unwrap()),
        )
        .init();

    let command = match matches.subcommand() {
        Some(("lock", sub_matches)) => Command::Lock {
            config_path: sub_matches
                .get_one::<String>("config")
                .expect("Default config path should exist")
                .clone(),
            lockfile_path: sub_matches
                .get_one::<String>("lockfile")
                .expect("Default lockfile path should exist")
                .clone(),
        },
        Some(("download", sub_matches)) => Command::Download {
            config_path: sub_matches
                .get_one::<String>("config")
                .expect("Default config path should exist")
                .clone(),
            lockfile_path: sub_matches
                .get_one::<String>("lockfile")
                .expect("Default lockfile path should exist")
                .clone(),
        },
        _ => {
            eprintln!("No subcommand provided. Use 'lock' or 'download'.");
            std::process::exit(1);
        }
    };

    Args { command, log_level }
}

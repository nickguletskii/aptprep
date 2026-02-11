use clap::{ArgAction, Parser, Subcommand};
use tracing::Level;
use tracing_subscriber;

#[derive(Debug, Clone)]
pub enum Command {
    Lock {
        config_path: String,
        lockfile_path: String,
        target_architectures: Vec<String>,
    },
    Download {
        config_path: Option<String>,
        lockfile_path: String,
        output_dir: Option<String>,
        max_concurrency_per_host: usize,
        max_retries: usize,
        download_parallelism: usize,
        checking_parallelism: usize,
    },
    GeneratePackagesFileFromLockfile {
        config_path: Option<String>,
        lockfile_path: String,
        output_path: Option<String>,
    },
}

pub struct Args {
    pub command: Command,
    pub log_level: Level,
}

#[derive(Debug, Parser)]
#[command(
    name = "aptprep",
    version,
    author = "Nick Guletskii",
    about = "Resolve all Debian package dependencies needed to install a given set of Debian packages behind an air gap"
)]
struct Cli {
    #[arg(
        short = 'v',
        long = "verbose",
        help = "Sets the level of verbosity",
        action = ArgAction::Count,
        global = true
    )]
    verbose: u8,

    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    /// Download package lists, resolve dependencies and create lockfile
    Lock {
        #[arg(
            short = 'c',
            long = "config",
            value_name = "FILE",
            help = "Sets a custom config file",
            default_value = "config.yaml"
        )]
        config: String,

        #[arg(
            short = 'l',
            long = "lockfile",
            value_name = "FILE",
            help = "Sets the output lockfile path",
            default_value = "aptprep.lock"
        )]
        lockfile: String,

        #[arg(
            short = 'a',
            long = "target-architecture",
            value_name = "ARCH",
            help = "Overrides target architectures (repeat or use comma-separated values)",
            action = ArgAction::Append,
            value_delimiter = ','
        )]
        target_architectures: Vec<String>,
    },

    /// Read lockfile and download all required packages
    Download {
        #[arg(
            short = 'c',
            long = "config",
            value_name = "FILE",
            help = "Optional config file for output-dir fallback and lockfile hash validation"
        )]
        config: Option<String>,

        #[arg(
            short = 'l',
            long = "lockfile",
            value_name = "FILE",
            help = "Sets the input lockfile path",
            default_value = "aptprep.lock"
        )]
        lockfile: String,

        #[arg(
            short = 'o',
            long = "output-dir",
            value_name = "DIR",
            help = "Overrides output directory for downloaded packages and generated Packages file"
        )]
        output_dir: Option<String>,

        #[arg(
            long = "max-concurrency-per-host",
            value_name = "N",
            help = "Maximum concurrent HTTP requests per host",
            default_value_t = 8
        )]
        max_concurrency_per_host: usize,

        #[arg(
            long = "max-retries",
            value_name = "N",
            help = "Maximum retry attempts for failed HTTP operations",
            default_value_t = 5
        )]
        max_retries: usize,

        #[arg(
            long = "download-parallelism",
            value_name = "N",
            help = "Maximum number of simultaneous downloads",
            default_value_t = 16
        )]
        download_parallelism: usize,

        #[arg(
            long = "checking-parallelism",
            value_name = "N",
            help = "Maximum number of concurrent file digest checks",
            default_value_t = 128
        )]
        checking_parallelism: usize,
    },

    /// Read lockfile and generate a Packages index file
    #[command(
        name = "generate_packages_file_from_lockfile",
        visible_alias = "generate-packages-file-from-lockfile"
    )]
    GeneratePackagesFileFromLockfile {
        #[arg(
            short = 'c',
            long = "config",
            value_name = "FILE",
            help = "Optional config file for output path fallback"
        )]
        config: Option<String>,

        #[arg(
            short = 'l',
            long = "lockfile",
            value_name = "FILE",
            help = "Sets the input lockfile path",
            default_value = "aptprep.lock"
        )]
        lockfile: String,

        #[arg(
            short = 'o',
            long = "output",
            value_name = "FILE",
            help = "Sets the output Packages file path (default: <config.output.path>/Packages)"
        )]
        output: Option<String>,
    },
}

pub fn parse_args() -> Args {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
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

    let command = match cli.command {
        CliCommand::Lock {
            config,
            lockfile,
            target_architectures,
        } => Command::Lock {
            config_path: config,
            lockfile_path: lockfile,
            target_architectures,
        },
        CliCommand::Download {
            config,
            lockfile,
            output_dir,
            max_concurrency_per_host,
            max_retries,
            download_parallelism,
            checking_parallelism,
        } => Command::Download {
            config_path: config,
            lockfile_path: lockfile,
            output_dir,
            max_concurrency_per_host,
            max_retries,
            download_parallelism,
            checking_parallelism,
        },
        CliCommand::GeneratePackagesFileFromLockfile {
            config,
            lockfile,
            output,
        } => Command::GeneratePackagesFileFromLockfile {
            config_path: config,
            lockfile_path: lockfile,
            output_path: output,
        },
    };

    Args { command, log_level }
}

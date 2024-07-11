//! The main structures and functions for the server.

use std::path::PathBuf;

use clap::{Parser};

use std::time::SystemTime;

/// A structure that holds the state of the entire server.
pub struct App {
    config: Config,

}

/// The current configuration for the server, e.g. 404 page path, number of threads in the pool,
/// and the like.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Config {
    ///The path to the HTML file when the requested URI is not found.
    #[arg(short, long, value_name="404_PAGE")]
    not_found_uri: Option<PathBuf>,
    
    ///Controls whether or not logs of type `Debug` should be displayed in stdout.
    #[arg(short, long, value_name="DBG")]
    show_debug_logs: bool,

    ///Controls whether or not logs should be output to stdout.
    #[arg(short, long, value_name="SHOW_LOGS")]
    show_logs_in_stdout: bool, 

    ///Specifies the maximum number of threads that are used while the server is running. Each
    ///thread is used for a single connection, however to protect against DDOS (Distrubted Denial
    ///of Service) attacks, this number is a finite amount.
    #[arg(short, long, value_name="NUM_THREADS")]
    num_threads: u64,
}

impl App {
    pub fn new(config: Config) -> App {
        log::info!("Initializing Ferric...");

        App {
            config: config,
        }
        //FIXME: integrate logging startup into this function
    }

    pub fn init_logging(&self) -> Result<(), fern::InitError> {
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{} {} {}] {}",
                    humantime::format_rfc3339_seconds(SystemTime::now()),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .level(match self.config.show_debug_logs {
                true => log::LevelFilter::Debug,
                false => log::LevelFilter::Info,
            })
            .chain(fern::log_file(format!("ferric_log_{}.log", humantime::format_rfc3339_seconds(SystemTime::now())))?)
            //FIXME: make this actually respect config.show_logs_in_stdout
            .chain(std::io::stdout())
            .apply()?;
        Ok(())
    }
}

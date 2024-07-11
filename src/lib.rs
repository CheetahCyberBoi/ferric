use log::{debug, error, info, trace, warn};
use std::time::SystemTime;



pub mod thread_pool;

///The entire state of the App.

pub struct App {
    pub client_thread_pool: thread_pool::ThreadPool,    

    pub config: Config,
}

///The configuration for Ferric, such as number of threadpool threads, paths to the root URI and
///404 page, and the like. 
pub struct Config {

    ///Controls whether logs of type `Debug` should be output.
    pub debug_logs: bool,

    ///Controls the maximum number of threads that will be used by the pool.
    pub num_threads_in_pool: usize,
}

impl App {
    pub fn new(config: Config) -> App {
        App { client_thread_pool: thread_pool::ThreadPool::new(config.num_threads_in_pool), config }

    }

    pub fn initialize_logging(&self) -> Result<(), fern::InitError> {
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "({} {}) [{}] {}",
                    humantime::format_rfc3339_seconds(SystemTime::now()),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .level(match self.config.debug_logs {
                true => log::LevelFilter::Debug,
                false => log::LevelFilter::Info,
            })
            .chain(std::io::stdout())
            .chain(fern::log_file(format!("ferric_log_{}.log", humantime::format_rfc3339_seconds(SystemTime::now())))?)
            .apply()?;
        Ok(())
    }
}

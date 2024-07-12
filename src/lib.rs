use std::{
    time::SystemTime,
    net::{SocketAddr, TcpListener, TcpStream},
    io::{prelude::*, BufReader},
};

use log::{info, debug, warn};
use httparse::{Request, Header};

pub mod thread_pool;

///The entire state of the App.

pub struct App {
    pub client_thread_pool: thread_pool::ThreadPool,    

    pub config: Config,
}

///The configuration for Ferric, such as number of threadpool threads, paths to the root URI and
///404 page, and the like. 
pub struct Config {

    ///Controls the port and IP ferric is exposed to.
    pub outfacing_ip: SocketAddr,

    ///Controls whether logs of type `Debug` should be output.
    pub debug_logs: bool,

    ///Controls the maximum number of threads that will be used by the pool.
    pub num_threads_in_pool: usize,

    ///The path to be used as the root URI.
    pub root_uri: std::path::PathBuf,
}

//NOTE: make these into config options in the future
const MAX_HEADERS: usize = 16;


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

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Running app...");

        //Bind to the Config's port.
        let str_ip = self.config.outfacing_ip.to_string();
        let tcp_listener = TcpListener::bind(&str_ip[..])?;
        info!("Bound to port {}, outfacing IP is: {}", self.config.outfacing_ip.port(), self.config.outfacing_ip.ip());

        //Iterate over incoming requests from clients.
        for stream in tcp_listener.incoming() {
            info!("Connection established!");
            let stream = stream.unwrap();

            self.handle_connection(stream);            
        }

        Ok(())



    }

    fn handle_connection(&self, mut stream: TcpStream) {
        self.client_thread_pool.execute(move || {
            //Parse the client's request into usable string data.
            let mut req_buf = stream.bytes().map(|res| res.unwrap()).collect::<Vec<u8>>();
            //TODO: figure out how to extract this logic into a different function without borrow
            //checker throwing a fit
            let mut headers: Vec<Header> = Vec::with_capacity(MAX_HEADERS);
            let mut req = Request::new(&mut headers);
            let res = req.parse(&req_buf);
            if res.expect("Failed to parse request!").is_partial() {
                match req.path {
                    Some(ref path) => {
                        //we have an actual path here, do the thingies
                        info!("Request parsed!");
                        debug!("Request is asking for file: {path}");
                    },
                    None => {
                        //gotta read more and parse again
                        warn!("Unable to fully parse request. Retrying...");
                    }
                }
            }

             
        });
    }


}

use std::{
    time::SystemTime,
    net::{SocketAddr, TcpListener, TcpStream},
    io::prelude::*,
    fs::*,
};

use log::{info, debug, warn};


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
    //The path to the HTML file meant to be shown when the requested resource is not found.
    pub not_found_path: Option<std::path::PathBuf>,
}



impl App {
    pub fn new(config: Config) -> App {
        App { client_thread_pool: thread_pool::ThreadPool::new(config.num_threads_in_pool), config }

    }
    ///Initializes the logging system for `ferric`.
    ///This returns an error if the logging system (currently Fern) fails to either apply the
    ///configuration or create a log_file
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
    ///Runs the `App`.
    ///# Errors:
    ///This method will return an `Error` type if binding to the specified port fails. For example,
    ///if `Config` specified that `outfacing_ip` is port 80, it would return `Error`, as that port
    ///is administrator priveledges only.
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

    fn handle_connection<'a>(&self, mut stream: TcpStream) {
        self.client_thread_pool.execute(move || {
            //This took an ungodly number of hours to get working.
            let raw_req_buf = App::read_request(&mut stream);
            //debug!("Raw request: {:#?}", String::from_utf8(raw_req_buf));
            
            //Parse the request into `httparse::Request`
            const MAX_HEADERS: usize = 16usize;
            let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
            let mut request = httparse::Request::new(&mut headers);
            let result = request.parse(&raw_req_buf).expect("Unable to parse request.");
            //We (hopefully) parsed the data, check to make sure it parsed fine
            match result {
                httparse::Status::Complete(_) => {
                   info!("Request parsed!");
                },
                httparse::Status::Partial => {
                    //FIXME: change this to do proper error handling soon.
                    eprintln!("Request partially parsed, unable to continue.");
                }
            }
            //Request is now parsed, we can begin the tomfoolery
            debug!("Request: {:#?}", &request);
            //Dig up the file they requested
            let mut requested_path = self.config.root_uri.clone();
            requested_path.push(request.path.unwrap());
            let  file_res = File::open(requested_path);
            let response: httparse::Response = match file_res {
                Ok(ref file) => {
                    //The file exists, shovel it back to the user
                    let mut buf = String::new();
                    file.read_to_string(&mut buf).unwrap();
                    httparse::Response {
                        version: Some(1),
                        code: Some(200),
                        reason: Some("OK"),
                        headers: &mut [httparse::Header{name: "Content-Length", value: buf.len().to_string().as_bytes()}]
                    }
                },
                Err(error) => {
                    //Check if it's a `NotFound` error, if so send back
                    match error.kind() {
                        std::io::ErrorKind::NotFound => {
                            warn!("Requested path {} not found", request.path.unwrap());
                            let not_found_file = match &self.config.not_found_path {
                                Some(path) => {
                                    File::open(path).expect("Provided \"not found\" HTML file not found.")
                                    //NOTE: find a way to make this into a recoverable error
                                },
                                None => {
                                    let include_path = env!("PWD").to_string() + "404.html";
                                    File::open(include_path).expect("Could not find default 404.html in the current directory.")
                                                                    
                                }
                            };
                            //Shovel this back.
                            httparse::Response {
                                version: Some(1), //for HTTP/1.1
                                code: Some(404), //because we didn't find anything
                                reason: Some("NOT FOUND"),
                                headers: &mut [httparse::EMPTY_HEADER],
                            }
                        },
                        //NOTE: add other variants like PermissionDenied and the like
                        _ => {warn!("Other error occured");
                            //TODO: find a better error code for this
                            httparse::Response {
                                version: Some(1),
                                code: Some(500),
                                reason: Some("INTERNAL SERVER ERROR"),
                                headers: &mut [httparse::EMPTY_HEADER],
                            }
                        }
                    }
                }
                
            };
        });
    }

    //Given to me by @flyingparrot225 on discord.
    fn read_request(stream: &mut TcpStream) -> Vec<u8> {
        let mut request_buffer = Vec::new();
        const BYTE_BUFFER_SIZE: usize = 256;
        let mut byte_buffer = [0u8; BYTE_BUFFER_SIZE];

        loop {
            let bytes_read = stream.read(&mut byte_buffer).unwrap();

            request_buffer.extend_from_slice(&byte_buffer[..bytes_read]);

            if bytes_read < BYTE_BUFFER_SIZE {
                break;
            }
        }

        request_buffer
    }


}

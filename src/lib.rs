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
        use fern::colors::{ColoredLevelConfig, Color};
        let colors = ColoredLevelConfig::new()
            .info(Color::Green)
            .warn(Color::Yellow)
            .error(Color::Red)
            .debug(Color::Blue);
        fern::Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "{color_line}[{date} {level} {target} {color_line}] {message}\x1B[0m",
                    color_line = format_args!(
                        "\x1B[{}m",
                        colors.get_color(&record.level()).to_fg_str()
                    ),
                    date = humantime::format_rfc3339_seconds(SystemTime::now()),
                    level = colors.color(record.level()),
                    target = record.target(),
                    message =message
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
            let mut stream = stream;
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

            debug!("Request from client: {:#?}", &request);
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

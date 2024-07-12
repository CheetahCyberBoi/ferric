use ferric::{App, Config};

fn main() {
    let mut app = App::new(Config {
        outfacing_ip: std::net::SocketAddr::V4(std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(127, 0, 0, 1), 7878)),
        debug_logs: true,
        num_threads_in_pool: 5usize,
        root_uri: std::path::PathBuf::new(),
    });
    app.initialize_logging().unwrap();
    log::info!("Logging system initialized!");

    app.run().unwrap_or_else(|err| eprintln!("ERR: Failed to start Ferric due to: {err}"));
}

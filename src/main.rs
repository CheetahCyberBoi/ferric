use ferric::{App, Config};

fn main() {
    let app = App::new(Config {
        debug_logs: true,
        num_threads_in_pool: 5usize,
    });
    app.initialize_logging().unwrap();
    log::info!("Logs are gaming!");
    for i in 0..5 {
        app.client_thread_pool.execute(move || {
            log::info!("thready mcthreadson numero {i} is existing...");
        })
    }
    
}

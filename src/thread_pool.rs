///A thread pool implementation, ripped straight from the Rust book
///(https://doc.rust-lang.org/book/ch20-02-multithreaded.html)

use std::{
    sync::{mpsc, Arc, Mutex},
    thread
};
/// A thread-pool that has a finite number of threads for execution.
pub struct ThreadPool {
    workers: Vec<Worker>,
    dispatch_sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    ///Instantiates a new `ThreadPool` with a given number of threads to use.
    pub fn new(num_threads: usize) -> ThreadPool {
        assert!(num_threads > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(num_threads);

        for id in 0..num_threads {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, dispatch_sender: sender }
    }
    ///Sends the given closure to the `ThreadPool`'s Workers to be executed.
    ///The closure must be able to be sent over a `mpsc::channel()` connection.
    pub fn execute<F>(&self, f: F)
    where 
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        
        self.dispatch_sender.send(job).expect("ERR: Failed to send job to worker threads!");

    }
}
///A struct that stores a worker's ID and currently running thread.
///This is used internally by the `ThreadPool` struct.
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    ///Instantiates a new Worker, with a given ID and a Arc-cloned `mpsc::Receiver`.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            //NOTE: see if i can perform some `and_then` tomfuckery here
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job, executing.");

            job();
        });

        Worker { id, thread }
    }
}



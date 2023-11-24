use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

/// ThreadPool struct represents a pool of worker threads.
/// It holds a vector of workers and a sender for sending jobs to workers.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

/// Job type alias represents a job that can be executed by the thread pool.
/// It is a boxed closure that can be called once and is sendable between threads.
type Job = Box<dyn FnOnce() + Send + 'static>;

/// PoolCreationError enum represents the possible errors that can occur when creating a thread pool.
#[derive(Debug)]
pub enum PoolCreationError {
    ZeroSizedPool,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size > 0 {
            let (sender, receiver) = mpsc::channel();

            let receiver = Arc::new(Mutex::new(receiver));

            let mut workers = Vec::with_capacity(size);

            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)));
            }

            Ok(
                ThreadPool {
                    workers,
                    sender: Some(sender)
                }
            )
        } else {
            Err(PoolCreationError::ZeroSizedPool)
        }
    }

    /// Execute a job in the thread pool.
    ///
    /// The job is a closure that is sendable between threads and can be called once.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    /// Clean up the thread pool.
    ///
    /// This function is called when the ThreadPool goes out of scope.
    /// It sends a terminate message to all workers and waits for them to finish.
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

/// Worker struct represents a worker in the thread pool.
///
/// It holds an id and a thread handle.
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Create a new Worker.
    ///
    /// The worker is created with a given id and a receiver for receiving jobs.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
use std::ops::Deref;
use std::sync::{Arc, LockResult, mpsc, Mutex};
use std::thread;
use std::time::Duration;

pub fn test() {
    println!("--- thread pool test ---");

    let thread_pool = ThreadPool::new(4);

    for i in 0..100 {
        let n = i;
        thread_pool.execute(move ||{
            for j in 0..n {
                println!("Task#{n} Step#{j}");
            }
        });
    }

    thread::sleep(Duration::from_millis(2000));
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}
impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, receiver.clone()));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() -> () + Send + 'static {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = match receiver.lock().expect("error getting lock").recv() {
                Ok(job ) => { job }
                Err(_) => { break }
            };
            println!("thread running job");
            job();
        });
        Worker {
            id,
            thread,
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

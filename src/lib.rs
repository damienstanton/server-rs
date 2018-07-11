//! naive multi-threaded server from chapter 20 of TRPL.
//! It is essentially 1:1 with the book version, albeit with weaks to documentation and naming conventions.
//! ## 🚧 TODOS:
//! - Get rid of `unwrap()` calls and enclose those in better types.
//! - Reorg into modules
//! - Docs

use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct WorkerPool {
    workers: Vec<Worker>,
    send_chan: mpsc::Sender<Message>,
}

impl WorkerPool {
    pub fn new(size: usize) -> WorkerPool {
        // TODO: refactor to Result<WorkerPool, PoolCreationError>
        // instead of the assertion
        assert!(size > 0);

        let (send_chan, rec_chan) = mpsc::channel();
        let rec_chan = Arc::new(Mutex::new(rec_chan));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&rec_chan)));
        }

        WorkerPool { workers, send_chan }
    }

    pub fn run<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.send_chan.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");
        for _ in &mut self.workers {
            self.send_chan.send(Message::Terminate).unwrap();
        }
        println!("Shutting down...");
        for worker in &mut self.workers {
            println!("Worker {} shutting down...", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

type Job = Box<FnBox + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, rec_chan: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = rec_chan.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job.call();
                }
                Message::Terminate => {
                    println!("Worker {} received terminate signal.", id);
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

trait FnBox {
    fn call(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call(self: Box<F>) {
        (*self)()
    }
}

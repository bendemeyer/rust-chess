use std::{thread::{self, JoinHandle}};

use crossbeam_channel::{Sender, Receiver, unbounded};


pub struct Job<T: Send + 'static> {
    pub task: Box<dyn FnOnce() -> T + Send>,
    pub comm: Sender<T>
}

impl<T: Send + 'static> Job<T> {
    pub fn run(self) {
        self.comm.send((self.task)()).unwrap();
    }
}


pub struct ThreadPool<T: Send + 'static> {
    queue_writer: Sender<Job<T>>,
    queue_reader: Receiver<Job<T>>,
    handles: Vec<JoinHandle<()>>,
}

unsafe impl<T: Send + 'static> Send for ThreadPool<T> {}
unsafe impl<T: Send + 'static> Sync for ThreadPool<T> {}

impl<T: Send + 'static> ThreadPool<T> {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        return Self {
            queue_writer: tx,
            queue_reader: rx,
            handles: Vec::new(),
        }
    }

    pub fn enqueue(&self, job: Job<T>) {
        self.queue_writer.send(job).expect("Failed enqueueing a job for the thread pool");
    }

    fn start_worker(&self) -> JoinHandle<()> {
        let queue = self.queue_reader.clone();
        return thread::spawn(move || {
            while let Ok(job) = queue.recv() {
                job.run()
            }
        })
    }

    pub fn init(&mut self, pool_size: u8) {
        self.handles = (0..pool_size).map(|_| {
            self.start_worker()
        }).collect();
    }

    pub fn join(self) {
        drop(self.queue_writer);
        self.handles.into_iter().for_each(|h| h.join().unwrap());
    }
}

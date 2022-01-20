use std::{hash::Hash, thread::{JoinHandle, self}, sync::Arc};

use crossbeam_channel::{Sender, Receiver, unbounded};

use super::{queues::{PriorityQueueWriter, PriorityQueueReader, PriorityQueueBuilder, QueueType}, tasks::{Task, AsyncTask}};

pub struct ThreadPool<T: Send + 'static> {
    queue_writer: Sender<Task<T>>,
    queue_reader: Receiver<Task<T>>,
    handles: Vec<JoinHandle<()>>,
}

unsafe impl<T: Send + 'static> Send for ThreadPool<T> {}
unsafe impl<T: Send + 'static> Sync for ThreadPool<T> {}

impl<T: Send + 'static> ThreadPool<T> {
    pub fn new() -> Self {
        let (tx, rx): (Sender<Task<T>>, Receiver<Task<T>>) = unbounded();
        return Self {
            queue_writer: tx,
            queue_reader: rx,
            handles: Vec::new(),
        }
    }

    pub fn enqueue(&self, job: Task<T>) {
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


pub struct AsyncPriorityThreadPool<P: Copy + Hash + Eq> {
    queue_writer: Arc<PriorityQueueWriter<P, AsyncTask>>,
    queue_reader: Arc<PriorityQueueReader<AsyncTask>>,
    handles: Vec<JoinHandle<()>>,
}

unsafe impl<P: Copy + Hash + Eq> Send for AsyncPriorityThreadPool<P> {}
unsafe impl<P: Copy + Hash + Eq> Sync for AsyncPriorityThreadPool<P> {}

impl<P: Copy + Hash + Eq> AsyncPriorityThreadPool<P> {
    pub fn from_builder(builder: PriorityQueueBuilder<P>) -> Self {
        let (writer, reader) = builder.build(QueueType::FIFO);
        return Self {
            queue_writer: Arc::new(writer),
            queue_reader: Arc::new(reader),
            handles: Vec::new(),
        }
    }

    pub fn clone_writer(&self) -> Arc<PriorityQueueWriter<P, AsyncTask>> {
        return Arc::clone(&self.queue_writer);
    }

    pub fn enqueue(&self, job: AsyncTask, priority: &P) {
        self.queue_writer.enqueue(job, priority).expect("Error enqueueing message in AsyncPriorityThreadPool");
    }

    fn start_worker(&self) -> JoinHandle<()> {
        let queue = Arc::clone(&self.queue_reader);
        return thread::spawn(move || {
            while let Ok(job) = queue.dequeue() {
                job.run();
            }
        })
    }

    pub fn init(&mut self, pool_size: u8) {
        self.handles = (0..pool_size).map(|_| {
            self.start_worker()
        }).collect();
    }

    pub fn join(self) {
        println!("{} References to the PriorityQueueWriter remain", Arc::strong_count(&self.queue_writer));
        Arc::try_unwrap(self.queue_writer).unwrap_or_else(|_| panic!("Error extracting PriorityQueueWriter from AsyncPriorityThreadPool for destruction")).destruct_queue();
        self.handles.into_iter().for_each(|h| h.join().unwrap());
    }
}

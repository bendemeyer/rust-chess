use std::{collections::VecDeque, thread::{self, JoinHandle}, sync::{Mutex, Arc, mpsc::{Sender, Receiver, channel}}};


pub struct Job<T: Send + 'static> {
    pub task: Box<dyn FnOnce() -> T + Send>,
    pub comm: Sender<T>
}

impl<T: Send + 'static> Job<T> {
    pub fn run(self) {
        self.comm.send((self.task)()).unwrap();
    }
}


pub struct QueuedThreadPool<T: Send + 'static> {
    queue_writer: Sender<Job<T>>,
    queue_reader: Arc<Mutex<Receiver<Job<T>>>>,
    handles: Vec<JoinHandle<()>>,
}

impl<T: Send + 'static> QueuedThreadPool<T> {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        return Self {
            queue_writer: tx,
            queue_reader: Arc::new(Mutex::new(rx)),
            handles: Vec::new(),
        }
    }

    pub fn enqueue(&self, job: Job<T>) {
        self.queue_writer.send(job).expect("Failed enqueueing a job for the thread pool");
    }

    fn start_worker(&self) -> JoinHandle<()> {
        let mutex = Arc::clone(&self.queue_reader);
        return thread::spawn(move || {
            loop {
                let job: Result<Job<T>, _>;
                {
                    let queue = mutex.lock().unwrap();
                    job = queue.recv();
                }
                match job {
                    Ok(j) => j.run(),
                    Err(_) => break,
                }
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


pub struct ThreadPool<F: Send + 'static, T: Send + 'static> where F: FnOnce() -> T {
    queue: Arc<Mutex<WorkQueue<F, T>>>,
    handles: Vec<JoinHandle<()>>,
}

impl<F: Send + 'static, T: Send + 'static> ThreadPool<F, T> where F: FnOnce() -> T {
    pub fn from_queue(queue: WorkQueue<F, T>) -> Self {
        return Self {
            queue: Arc::new(Mutex::new(queue)),
            handles: Vec::new(),
        }
    }

    fn start_worker(&mut self) -> JoinHandle<()> {
        let mutex = Arc::clone(&self.queue);
        return thread::spawn(move || {
            loop {
                let task: Option<F>;
                {
                    let mut queue = mutex.lock().unwrap();
                    task = queue.dequeue();
                }
                match task {
                    Some(t) => {
                        let result = t();
                        {
                            let mut queue = mutex.lock().unwrap();
                            queue.add_result(result);
                        }
                    },
                    None => break,
                }
            }
        })
    }

    pub fn run(&mut self, pool_size: u8) {
        self.handles = (0..pool_size).map(|_| {
            self.start_worker()
        }).collect();
    }

    pub fn join(self) -> Vec<T> {
        self.handles.into_iter().for_each(|h| { h.join().unwrap(); });
        return Arc::try_unwrap(self.queue).unwrap_or_else(|_| {panic!("Error getting results from thread pool")}).into_inner().unwrap().into_results();
    }
}


#[derive(Debug)]
pub struct WorkQueue<F: Send + 'static, T: Send + 'static> where F: FnOnce() -> T {
    queue: VecDeque<F>,
    results: Vec<T>,
}

impl<F: Send + 'static, T: Send + 'static> WorkQueue<F, T> where F: FnOnce() -> T {
    pub fn from_iter<I>(iter: I) -> Self where I: Iterator<Item=F> {
        return Self {
            queue: iter.collect(),
            results: Vec::new(),
        }
    }

    pub fn enqueue(&mut self, task: F) {
        self.queue.push_back(task);
    }

    pub fn dequeue(&mut self) -> Option<F> {
        return self.queue.pop_front();
    }

    pub fn add_result(&mut self, result: T) {
        self.results.push(result);
    }

    pub fn get_results(&self) -> &Vec<T> {
        return &self.results;
    }

    fn into_results(self) -> Vec<T> {
        return self.results;
    }
}
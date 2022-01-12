use std::{collections::VecDeque, thread::{self, JoinHandle}, sync::{Mutex, Arc, mpsc::Sender, RwLock}, time::Duration};


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
    queue: Arc<Mutex<VecDeque<Job<T>>>>,
    handles: Vec<JoinHandle<()>>,
    terminated: Arc<RwLock<bool>>,
}

impl<T: Send + 'static> QueuedThreadPool<T> {
    pub fn new() -> Self {
        return Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            handles: Vec::new(),
            terminated: Arc::new(RwLock::new(false)),
        }
    }

    pub fn enqueue(&mut self, job: Job<T>) {
        self.queue.lock().unwrap().push_back(job);
    }

    pub fn enqueue_many<I>(&mut self, jobs: I) where I: Iterator<Item=Job<T>> {
        self.queue.lock().unwrap().extend(jobs);
    }

    fn start_worker(&mut self) -> JoinHandle<()> {
        let queue = Arc::clone(&self.queue);
        let terminated = Arc::clone(&self.terminated);
        return thread::spawn(move || {
            loop {
                let job = queue.lock().unwrap().pop_front();
                match job {
                    Some(j) => { j.run(); },
                    None => {
                        if *terminated.read().unwrap() { break; }
                        thread::sleep(Duration::from_millis(10))
                    },
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
        *self.terminated.write().unwrap() = true;
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
use std::{collections::VecDeque, hash::Hash, thread::{self, Thread}};

use crossbeam_channel::{Sender, Receiver, unbounded, RecvError, TryRecvError, SendError};
use fxhash::FxHashMap;
use lockfree::prelude::Stack;


pub struct SimpleQueue<T> {
    input: Sender<T>,
    output: Receiver<T>,
}

impl<T> Clone for SimpleQueue<T> {
    fn clone(&self) -> Self {
        return Self {
            input: self.input.clone(),
            output: self.output.clone(),
        }
    }
}

impl<T> SimpleQueue<T> {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        return Self {
            input: tx,
            output: rx,
        }
    }

    pub fn enqueue(&self, message: T) -> Result<(), SendError<T>> {
        self.input.send(message)
    }

    pub fn dequeue(&self) -> Result<T, RecvError> {
        self.output.recv()
    }

    pub fn try_dequeue(&self) -> Result<T, TryRecvError> {
        self.output.try_recv()
    }
}


pub enum QueueType {
    FIFO,
    LIFO,
}


pub struct PriorityQueueWriter<P: Copy + Hash + Eq, T> {
    priorities: Vec<P>,
    queues: FxHashMap<P, Sender<T>>,
    parked_threads: Receiver<Thread>,
}

impl<P: Copy + Hash + Eq, T> Clone for PriorityQueueWriter<P, T> {
    fn clone(&self) -> Self {
        return Self {
            priorities: self.priorities.clone(),
            queues: self.priorities.iter().map(|p| { (*p, self.queues.get(p).unwrap().clone()) }).collect(),
            parked_threads: self.parked_threads.clone(),
        }
    }
}

impl<P: Copy + Hash + Eq, T> PriorityQueueWriter<P, T> {
    pub fn new(rx: Receiver<Thread>) -> Self {
        return Self {
            priorities: Vec::new(),
            queues: Default::default(),
            parked_threads: rx,
        }
    }

    fn add_priority_queue(&mut self, priority: P, queue: Sender<T>) {
        self.priorities.push(priority);
        self.queues.insert(priority, queue);
    }

    pub fn enqueue(&self, message: T, priority: &P) -> Result<(), SendError<T>> {
        if let Some(queue) = self.queues.get(priority) {
            return match queue.send(message) {
                Ok(()) => {
                    if let Ok(t) = self.parked_threads.try_recv() {
                        t.unpark();
                    }
                    Ok(())
                },
                Err(se) => Err(se),
            }
        } else {
            Err(SendError(message))
        }
    }

    pub fn destruct_queue(mut self) {
        for priority in &self.priorities {
            let queue = self.queues.remove(priority).unwrap();
            drop(queue);
        }
        while let Ok(t) = self.parked_threads.try_recv() {
            t.unpark();
        }
    }
}


pub struct PriorityQueueReader<T> {
    fifo_queues: Vec<Receiver<T>>,
    lifo_queues: Vec<Stack<T>>,
    parked_threads: Sender<Thread>,
}

impl<T> PriorityQueueReader<T> {
    pub fn new(tx: Sender<Thread>) -> Self {
        return Self {
            fifo_queues: Default::default(),
            lifo_queues: Default::default(),
            parked_threads: tx,
        }
    }

    fn add_queue(&mut self, queue: Receiver<T>) {
        self.fifo_queues.push(queue);
    }

    pub fn dequeue(&self) -> Result<T, RecvError> {
        loop {
            match self.try_dequeue() {
                Ok(message) => return Ok(message),
                Err(TryRecvError::Disconnected) => return Err(RecvError),
                Err(TryRecvError::Empty) => {
                    self.parked_threads.send(thread::current()).expect("Error parking PriorityQueueReader thread.");
                    thread::park();
                }
            }
        }
    }

    pub fn try_dequeue(&self) -> Result<T, TryRecvError> {
        let mut disconnect_count = 0;
        for queue in &self.fifo_queues {
            match queue.try_recv() {
                Ok(msg) => return Ok(msg),
                Err(TryRecvError::Disconnected) => disconnect_count += 1,
                Err(TryRecvError::Empty) => (),
            }
        }
        if disconnect_count == self.fifo_queues.len() {
            return Err(TryRecvError::Disconnected);
        } else {
            return Err(TryRecvError::Empty);
        }
    }
}


pub struct PriorityQueueBuilder<P: Copy + Hash + Eq> {
    priorities: VecDeque<P>
}

impl<P: Copy + Hash + Eq> PriorityQueueBuilder<P> {
    pub fn new() -> Self {
        Self {
            priorities: Default::default(),
        }
    }

    pub fn from_priorities(priorities: Vec<P>) -> Self {
        return Self {
            priorities: priorities.into_iter().collect(),
        }
    }

    pub fn add_low_priority(mut self, priority: P) -> Self {
        self.priorities.push_back(priority);
        return self;
    }

    pub fn add_high_priority(mut self, priority: P) -> Self {
        self.priorities.push_front(priority);
        return self;
    }

    pub fn build<T>(self, _queue_type: QueueType) -> (PriorityQueueWriter<P, T>, PriorityQueueReader<T>) {
        let (tx, rx) = unbounded();
        let mut writer = PriorityQueueWriter::new(rx);
        let mut reader = PriorityQueueReader::new(tx);
        for priority in self.priorities {
            let (tx, rx) = unbounded();
            writer.add_priority_queue(priority, tx);
            reader.add_queue(rx);
        }
        return (writer, reader);
    }
}

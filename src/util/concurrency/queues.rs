use std::{collections::VecDeque, hash::Hash, thread::{self, Thread}};

use crossbeam_channel::{Sender, Receiver, unbounded, RecvError, TryRecvError, SendError};
use fxhash::FxHashMap;

use super::channels::{LifoSender, LifoReceiver, lifo_channel};


pub enum QueueType {
    FIFO,
    LIFO,
}

impl QueueType {
    fn build<T>(&self) -> (QueueWriter<T>, QueueReader<T>) {
        return match self {
            QueueType::FIFO => {
                let (tx, rx) = FifoQueueBuilder::build::<T>();
                (QueueWriter::FIFO(tx), QueueReader::FIFO(rx))
            },
            QueueType::LIFO => {
                let (tx, rx) = LifoQueueBuilder::build::<T>();
                (QueueWriter::LIFO(tx), QueueReader::LIFO(rx))
            }
        }
    }
}


enum QueueWriter<T> {
    FIFO(FifoQueueWriter<T>),
    LIFO(LifoQueueWriter<T>),
}

impl<T> QueueWriter<T> {
    fn enqueue(&self, message: T) -> Result<(), SendError<T>> {
        return match self {
            QueueWriter::FIFO(writer) => writer.enqueue(message),
            QueueWriter::LIFO(writer) => writer.enqueue(message),
        }
    }
}


enum QueueReader<T> {
    FIFO(FifoQueueReader<T>),
    LIFO(LifoQueueReader<T>),
}

impl<T> QueueReader<T> {
    pub fn dequeue(&self) -> Result<T, RecvError> {
        return match self {
            QueueReader::FIFO(reader) => reader.dequeue(),
            QueueReader::LIFO(reader) => reader.dequeue(),
        }
    }

    pub fn try_dequeue(&self) -> Result<T, TryRecvError> {
        return match self {
            QueueReader::FIFO(reader) => reader.try_dequeue(),
            QueueReader::LIFO(reader) => reader.try_dequeue(),
        }
    }
}


pub struct FifoQueueBuilder {}

impl FifoQueueBuilder {
    pub fn build<T>() -> (FifoQueueWriter<T>, FifoQueueReader<T>) {
        let (tx, rx) = unbounded();
        return (FifoQueueWriter { queue: tx }, FifoQueueReader { queue: rx })
    }
}

pub struct FifoQueueWriter<T> {
    queue: Sender<T>,
}

impl<T> Clone for FifoQueueWriter<T> {
    fn clone(&self) -> Self {
        return Self {
            queue: self.queue.clone(),
        }
    }
}

impl<T> FifoQueueWriter<T> {
    pub fn enqueue(&self, message: T) -> Result<(), SendError<T>> {
        self.queue.send(message)
    }
}

pub struct FifoQueueReader<T> {
    queue: Receiver<T>,
}

impl<T> Clone for FifoQueueReader<T> {
    fn clone(&self) -> Self {
        return Self {
            queue: self.queue.clone(),
        }
    }
}

impl<T> FifoQueueReader<T> {
    pub fn dequeue(&self) -> Result<T, RecvError> {
        self.queue.recv()
    }

    pub fn try_dequeue(&self) -> Result<T, TryRecvError> {
        self.queue.try_recv()
    }
}


pub struct LifoQueueBuilder {}

impl LifoQueueBuilder {
    pub fn build<T>() -> (LifoQueueWriter<T>, LifoQueueReader<T>) {
        let (tx, rx) = lifo_channel::<T>();
        return (LifoQueueWriter { queue: tx }, LifoQueueReader { queue: rx });
    }
}

pub struct LifoQueueWriter<T> {
    queue: LifoSender<T>,
}

impl<T> Clone for LifoQueueWriter<T> {
    fn clone(&self) -> Self {
        return Self {
            queue: self.queue.clone(),
        }
    }
}

impl<T> LifoQueueWriter<T> {
    pub fn enqueue(&self, message: T) -> Result<(), SendError<T>> {
        self.queue.send(message)
    }
}

pub struct LifoQueueReader<T> {
    queue: LifoReceiver<T>,
}

impl<T> Clone for LifoQueueReader<T> {
    fn clone(&self) -> Self {
        return Self {
            queue: self.queue.clone(),
        }
    }
}

impl<T> LifoQueueReader<T> {
    pub fn dequeue(&self) -> Result<T, RecvError> {
        self.queue.recv()
    }

    pub fn try_dequeue(&self) -> Result<T, TryRecvError> {
        self.queue.try_recv()
    }
}



pub struct PriorityQueueWriter<P: Copy + Hash + Eq, T> {
    priorities: Vec<P>,
    queues: FxHashMap<P, QueueWriter<T>>,
    parked_threads: Receiver<Thread>,
}

impl<P: Copy + Hash + Eq, T> Clone for PriorityQueueWriter<P, T> {
    fn clone(&self) -> Self {
        return Self {
            priorities: self.priorities.clone(),
            queues: self.priorities.iter().map(|p| {
                (*p, match self.queues.get(p).unwrap() {
                    QueueWriter::FIFO(w) => QueueWriter::FIFO(w.clone()),
                    QueueWriter::LIFO(w) => QueueWriter::LIFO(w.clone()),
                })
            }).collect(),
            parked_threads: self.parked_threads.clone(),
        }
    }
}

impl<P: Copy + Hash + Eq, T> PriorityQueueWriter<P, T> {
    pub fn enqueue(&self, message: T, priority: &P) -> Result<(), SendError<T>> {
        if let Some(queue) = self.queues.get(priority) {
            return match queue.enqueue(message) {
                Ok(_) => {
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

    pub fn destruct_queue(&mut self) {
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
    queues: Vec<QueueReader<T>>,
    parked_threads: Sender<Thread>,
}

impl<T> PriorityQueueReader<T> {
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
        for queue in &self.queues {
            match queue.try_dequeue() {
                Ok(msg) => return Ok(msg),
                Err(TryRecvError::Disconnected) => disconnect_count += 1,
                Err(TryRecvError::Empty) => (),
            }
        }
        if disconnect_count == self.queues.len() {
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

    pub fn build<T>(self, queue_type: QueueType) -> (PriorityQueueWriter<P, T>, PriorityQueueReader<T>) {
        let (parking_tx, parking_rx) = unbounded();
        let mut priorities = Vec::new();
        let mut writer_queues: FxHashMap<P, QueueWriter<T>> = Default::default();
        let mut reader_queues = Vec::new();
        for priority in self.priorities {
            let (tx, rx) = queue_type.build::<T>();
            priorities.push(priority);
            writer_queues.insert(priority, tx);
            reader_queues.push(rx);
        }
        let writer = PriorityQueueWriter {
            priorities: priorities,
            queues: writer_queues,
            parked_threads: parking_rx
        };
        let reader = PriorityQueueReader {
            queues: reader_queues,
            parked_threads: parking_tx
        };
        return (writer, reader);
    }
}

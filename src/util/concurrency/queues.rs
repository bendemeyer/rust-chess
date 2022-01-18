use std::{collections::VecDeque, hash::Hash, time::Duration, thread};

use crossbeam_channel::{Sender, Receiver, unbounded, RecvError, TryRecvError, SendError};
use fxhash::FxHashMap;


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


pub struct PriorityQueueWriter<P: Copy + Hash + Eq, T> {
    queues: FxHashMap<P, Sender<T>>,
}

impl<P: Copy + Hash + Eq, T> Clone for PriorityQueueWriter<P, T> {
    fn clone(&self) -> Self {
        return Self {
            queues: self.queues.iter().map(|(p, tx)| { (*p, tx.clone()) }).collect(),
        }
    }
}

impl<P: Copy + Hash + Eq, T> PriorityQueueWriter<P, T> {
    pub fn new() -> Self {
        return Self {
            queues: Default::default(),
        }
    }

    pub fn add_priority_queue(&mut self, priority: P, queue: Sender<T>) {
        self.queues.insert(priority, queue);
    }

    pub fn enqueue(&self, message: T, priority: &P) -> Result<(), SendError<T>> {
        if let Some(queue) = self.queues.get(priority) {
            return queue.send(message);
        } else {
            Err(SendError(message))
        }
    }
}


pub struct PriorityQueueReader<T> {
    queues: Vec<Receiver<T>>,
}

impl<T> Clone for PriorityQueueReader<T> {
    fn clone(&self) -> Self {
        return Self {
            queues: self.queues.iter().map(|rx| rx.clone()).collect(),
        }
    }
}

impl<T> PriorityQueueReader<T> {
    pub fn new() -> Self {
        return Self {
            queues: Default::default(),
        }
    }

    pub fn add_queue(&mut self, queue: Receiver<T>) {
        self.queues.push(queue);
    }

    pub fn dequeue(&self) -> Result<T, RecvError> {
        loop {
            match self.try_dequeue() {
                Ok(message) => return Ok(message),
                Err(TryRecvError::Disconnected) => return Err(RecvError),
                Err(TryRecvError::Empty) => thread::sleep(Duration::from_millis(10)),
            }
        }
    }

    pub fn try_dequeue(&self) -> Result<T, TryRecvError> {
        let mut disconnect_count = 0;
        for queue in &self.queues {
            match queue.try_recv() {
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

    pub fn build<T>(self) -> (PriorityQueueWriter<P, T>, PriorityQueueReader<T>) {
        let mut writer = PriorityQueueWriter::new();
        let mut reader = PriorityQueueReader::new();
        for priority in self.priorities {
            let (tx, rx) = unbounded();
            writer.add_priority_queue(priority, tx);
            reader.add_queue(rx);
        }
        return (writer, reader);
    }
}

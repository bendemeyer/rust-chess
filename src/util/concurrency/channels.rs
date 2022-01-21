use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc}, thread::{Thread, self}};

use crossbeam::channel::{SendError, RecvError, TryRecvError, Receiver, Sender, unbounded};
use lockfree::prelude::Stack;


pub fn lifo_channel<T>() -> (LifoSender<T>, LifoReceiver<T>) {
    let (parking_tx, parking_rx) = unbounded();
    let ch = LifoChannel::<T> {
        stack: Stack::new(),
        sender_count: AtomicUsize::new(1),
        receiver_count: AtomicUsize::new(1),
        parked_threads: parking_rx,
    };
    let channel_ref = Arc::new(ch);
    let tx = LifoSender::<T> {
        channel: Arc::clone(&channel_ref),
    };
    let rx = LifoReceiver::<T> {
        channel: Arc::clone(&channel_ref),
        parker: parking_tx,
    };
    return (tx, rx);
}


struct LifoChannel<T> {
    stack: Stack<T>,
    sender_count: AtomicUsize,
    receiver_count: AtomicUsize,
    parked_threads: Receiver<Thread>,
}

impl<T> LifoChannel<T> {
    fn push(&self, data: T) -> Result<(), SendError<T>> {
        if self.receiver_count.load(Ordering::Acquire) > 0 {
            self.stack.push(data);
            self.unpark();
            return Ok(());
        } else {
            return Err(SendError(data));
        }
    }

    fn pop(&self) -> Result<T, TryRecvError> {
        if self.sender_count.load(Ordering::Acquire) > 0 {
            return match self.stack.pop() {
                Some(data) => Ok(data),
                None => Err(TryRecvError::Empty),
            }
        } else {
            return Err(TryRecvError::Disconnected);
        }
    }

    fn unpark(&self) {
        if let Ok(t) = self.parked_threads.try_recv() {
            t.unpark();
        }
    }

    fn unpark_all(&self) {
        while let Ok(t) = self.parked_threads.try_recv() {
            t.unpark();
        }
    }

    fn clone_sender(&self) {
        self.sender_count.fetch_add(1, Ordering::Release);
    }

    fn clone_receiver(&self) {
        self.receiver_count.fetch_add(1, Ordering::Release);
    }

    fn destruct_sender(&self) {
        let prev = self.sender_count.fetch_sub(1, Ordering::Release);
        if prev <= 1 {
            self.unpark_all();
        }
    }

    fn destruct_receiver(&self) {
        self.receiver_count.fetch_sub(1, Ordering::Release);
    }
}

pub struct LifoSender<T> {
    channel: Arc<LifoChannel<T>>,
}

impl<T> Clone for LifoSender<T> {
    fn clone(&self) -> Self {
        self.channel.clone_sender();
        return Self {
            channel: Arc::clone(&self.channel),
        }
    }
}

impl<T> Drop for LifoSender<T> {
    fn drop(&mut self) {
        self.channel.destruct_sender();
    }
}

impl<T> LifoSender<T> {
    pub fn send(&self, data: T) -> Result<(), SendError<T>> {
        return self.channel.push(data);
    }
}

pub struct LifoReceiver<T> {
    channel: Arc<LifoChannel<T>>,
    parker: Sender<Thread>,
}

impl<T> Clone for LifoReceiver<T> {
    fn clone(&self) -> Self {
        self.channel.clone_receiver();
        return Self {
            channel: Arc::clone(&self.channel),
            parker: self.parker.clone(),
        }
    }
}

impl<T> Drop for LifoReceiver<T> {
    fn drop(&mut self) {
        self.channel.destruct_receiver();
    }
}

impl<T> LifoReceiver<T> {
    pub fn recv(&self) -> Result<T, RecvError> {
        loop {
            match self.try_recv() {
                Ok(data) => return Ok(data),
                Err(TryRecvError::Disconnected) => return Err(RecvError),
                Err(TryRecvError::Empty) => {
                    self.parker.send(thread::current()).expect("Error parking PriorityQueueReader thread.");
                    thread::park();
                }
            }
        }
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        return self.channel.pop();
    }
}

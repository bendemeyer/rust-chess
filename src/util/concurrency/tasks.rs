use crossbeam_channel::Sender;


pub struct Task<T: Send + 'static> {
    pub task: Box<dyn FnOnce() -> T + Send>,
    pub comm: Sender<T>
}

impl<T: Send + 'static> Task<T> {
    pub fn run(self) {
        self.comm.send((self.task)()).unwrap();
    }
}



pub struct AsyncTask {
    pub task: Box<dyn FnOnce() + Send>,
}

impl AsyncTask {
    pub fn run(self) {
        (self.task)()
    }
}
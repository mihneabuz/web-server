use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use log::info;

type Job = Box<dyn FnOnce() + Send>;

enum Task {
    New(Job),
    Exit,
}

struct Worker {
    id: i32,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: i32, receiver: Arc<Mutex<mpsc::Receiver<Task>>>) -> Self {
        let thread = Some(thread::spawn(move || loop {
            let task = { receiver.lock().unwrap().recv().unwrap() };

            match task {
                Task::New(job) => {
                    info!("Worker {} received job", id);
                    job();
                }
                Task::Exit => break,
            }
        }));

        Self { id, thread }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Task>,
}

impl ThreadPool {
    pub fn new(n: i32) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(n as usize);
        for i in 0..n {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + 'static + Send,
    {
        self.sender.send(Task::New(Box::new(f))).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Task::Exit).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                info!("Joining thread {}", &worker.id);
                thread.join().unwrap();
            }
        }
    }
}

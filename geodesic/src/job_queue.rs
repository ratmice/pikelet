use std::collections::VecDeque;
use std::sync::mpsc::Sender;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Job<Id, Data> {
    pub id: Id,
    pub data: Data,
}

impl<Id, Data> Job<Id, Data> {
    pub fn new(id: Id, data: Data) -> Job<Id, Data> {
        Job { id: id, data: data }
    }
}

struct JobQueue<Id, Data> {
    queued_jobs: VecDeque<Job<Id, Data>>,
}

impl<Id: PartialEq, Data> JobQueue<Id, Data> {
    fn new() -> JobQueue<Id, Data> {
        JobQueue {
            queued_jobs: VecDeque::new(),
        }
    }

    fn pop_front(&mut self) -> Option<Job<Id, Data>> {
        self.queued_jobs.pop_front()
    }

    fn push_back(&mut self, job: Job<Id, Data>) -> Option<Job<Id, Data>> {
        use std::mem;

        for queued_job in &mut self.queued_jobs {
            if queued_job.id == job.id {
                // Should we replace the queued job in its current positions,
                // or remove the queued job and push the new one to the back of
                // the queue?
                return Some(mem::replace(queued_job, job));
            }
        }

        self.queued_jobs.push_back(job);

        None
    }
}

pub fn spawn<Id, Data, F>(mut f: F) -> Sender<Job<Id, Data>> where
    Id: PartialEq + Send + 'static,
    Data: Send + 'static,
    F: FnMut(Job<Id, Data>) + Send + 'static,
{
    use std::sync::{Arc, Mutex};
    use std::sync::mpsc;
    use std::thread;

    let (job_tx, job_rx) = mpsc::channel();
    let queue = Arc::new(Mutex::new(JobQueue::new()));

    {
        let queue = queue.clone();

        thread::spawn(move || {
            for job in job_rx.iter() {
                let mut queue = queue.lock().unwrap();
                queue.push_back(job);
            }
        });
    }

    thread::spawn(move || {
        loop {
            let job = {
                // new scope to make sure that we drop the lock guard immediately
                let mut queue = queue.lock().unwrap();
                queue.pop_front()
            };

            if let Some(job) = job { f(job) };

            // Better way? The other thread could be something like an OTP gen_server...
            thread::sleep(Duration::from_millis(10));
        }
    });

    job_tx
}

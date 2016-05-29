use std::collections::VecDeque;
use std::sync::mpsc::Sender;
use std::time::Duration;

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[cfg(test)]
mod test {
    use super::{Job, JobQueue};
    use expectest::prelude::*;

    #[test]
    fn test_pop_front_empty() {
        let mut job_queue: JobQueue<(), ()> = JobQueue::new();
        expect!(job_queue.pop_front()).to(be_none());
    }

    #[test]
    fn test_push_back() {
        let mut job_queue = JobQueue::new();

        let job0 = Job::new(0, "0");
        let job1a = Job::new(1, "1a");
        let job1b = Job::new(1, "1b");
        let job2 = Job::new(2, "2");

        expect!(job_queue.push_back(job0)).to(be_none());
        expect!(job_queue.push_back(job1a)).to(be_none());
        expect!(job_queue.push_back(job2)).to(be_none());
        expect!(job_queue.push_back(job1b)).to(be_some().value(job1a));
    }

    #[test]
    fn test_first_in_first_out() {
        let mut job_queue = JobQueue::new();

        let job1 = Job::new(1, "1");
        let job2 = Job::new(2, "2");
        let job0 = Job::new(0, "0");

        job_queue.push_back(job1);
        job_queue.push_back(job2);
        job_queue.push_back(job0);

        expect!(job_queue.pop_front()).to(be_some().value(job1));
        expect!(job_queue.pop_front()).to(be_some().value(job2));
        expect!(job_queue.pop_front()).to(be_some().value(job0));
    }

    #[test]
    fn test_spawn() {
        use std::sync::mpsc;

        let job0 = Job::new(0, "0");
        let job1 = Job::new(1, "1");
        let job2 = Job::new(2, "2");

        let (tx, rx) = mpsc::channel();

        let job_tx = super::spawn(move |job| {
            tx.send(job).unwrap();
        });

        job_tx.send(job0).unwrap();
        job_tx.send(job1).unwrap();
        job_tx.send(job2).unwrap();

        expect!(rx.recv()).to(be_ok().value(job0));
        expect!(rx.recv()).to(be_ok().value(job1));
        expect!(rx.recv()).to(be_ok().value(job2));
    }
}

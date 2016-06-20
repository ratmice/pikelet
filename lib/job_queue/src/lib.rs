#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;

use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

struct JobQueue<Job> {
    queued_jobs: VecDeque<Job>,
}

impl<Job: PartialEq> JobQueue<Job> {
    fn new() -> JobQueue<Job> {
        JobQueue {
            queued_jobs: VecDeque::new(),
        }
    }

    fn pop_front(&mut self) -> Option<Job> {
        self.queued_jobs.pop_front()
    }

    fn push_back(&mut self, job: Job) -> Option<Job> {
        use std::mem;

        for queued_job in &mut self.queued_jobs {
            if queued_job == &job {
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

pub fn spawn<T, Job, F>(mut f: F) -> (Sender<Job>, Receiver<T>) where
    T: Send + 'static,
    Job: Send + PartialEq + 'static,
    F: FnMut(Job) -> T + Send + 'static,
{
    use std::sync::{Arc, Mutex};
    use std::sync::mpsc;
    use std::thread;

    let (job_tx, job_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
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

            if let Some(job) = job {
                if result_tx.send(f(job)).is_err() { break };
            };

            // Better way? The other thread could be something like an OTP gen_server...
            thread::sleep(Duration::from_millis(10));
        }
    });

    (job_tx, result_rx)
}

#[cfg(test)]
mod test {
    use super::JobQueue;
    use expectest::prelude::*;

    #[derive(Copy, Clone, Debug)]
    struct Job(usize, &'static str);

    impl PartialEq for Job {
        fn eq(&self, other: &Job) -> bool {
            self.0 == other.0
        }
    }

    #[test]
    fn test_pop_front_empty() {
        let mut job_queue: JobQueue<Job> = JobQueue::new();
        expect!(job_queue.pop_front()).to(be_none());
    }

    #[test]
    fn test_push_back() {
        let mut job_queue = JobQueue::new();

        let job0 = Job(0, "0");
        let job1a = Job(1, "1a");
        let job1b = Job(1, "1b");
        let job2 = Job(2, "2");

        expect!(job_queue.push_back(job0)).to(be_none());
        expect!(job_queue.push_back(job1a)).to(be_none());
        expect!(job_queue.push_back(job2)).to(be_none());
        expect!(job_queue.push_back(job1b)).to(be_some().value(job1a));
    }

    #[test]
    fn test_first_in_first_out() {
        let mut job_queue = JobQueue::new();

        let job1 = Job(1, "1");
        let job2 = Job(2, "2");
        let job0 = Job(0, "0");

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

        let job0 = Job(0, "0");
        let job1 = Job(1, "1");
        let job2 = Job(2, "2");

        let (tx, rx) = mpsc::channel();

        let (job_tx, result_rx) = super::spawn(|job| job);

        job_tx.send(job0).unwrap();
        job_tx.send(job1).unwrap();
        job_tx.send(job2).unwrap();

        expect!(result_rx.recv()).to(be_ok().value(job0));
        expect!(result_rx.recv()).to(be_ok().value(job1));
        expect!(result_rx.recv()).to(be_ok().value(job2));
    }
}

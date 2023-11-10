use crate::asset_management::ToUuid;
use anyhow::Result;
use crossbeam::queue::SegQueue;
use lazy_static::lazy_static;
use log::{info, warn};
use parking_lot::{Condvar, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use wgpu::{Device, Queue};

// TODO: Priority levels for jobs?

pub trait Job: Sync + Send + ToUuid {
    fn get_freq(&self) -> JobFrequency;
    fn run(&mut self, device: &Device, queue: &Queue) -> anyhow::Result<()>;
}

lazy_static! {
    static ref JOB_SCHEDULER: Mutex<JobScheduler> = Mutex::new(JobScheduler::init());
}

#[derive(Debug, Eq, PartialEq)]
pub enum JobFrequency {
    Frame,
    Periodically,
    Once,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum JobState {
    Queued = 0,
    Processing = 1,
    Failed = 2,
    Succeeded = 3,
}

pub struct JobStateTracker {
    state: Arc<AtomicU8>,
    condvar: Arc<(Condvar, Mutex<()>)>,
}

impl JobStateTracker {
    pub fn state(&self) -> JobState {
        match self.state.load(Ordering::Relaxed) {
            0 => JobState::Queued,
            1 => JobState::Processing,
            2 => JobState::Failed,
            3 => JobState::Succeeded,
            x => panic!("Invalid JobState: {}", x),
        }
    }

    pub fn flush(&self) -> Result<(), ()> {
        self.condvar.0.wait(&mut self.condvar.1.lock());

        loop {
            if self.state() == JobState::Succeeded {
                return Ok(());
            }
            if self.state() == JobState::Failed {
                return Err(());
            }
        }
    }
}

impl Clone for JobStateTracker {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            condvar: Arc::clone(&self.condvar),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ThreadState {
    Idle = 0,
    Processing = 1,
}

pub struct ThreadStateTracker {
    inner: Arc<AtomicU8>,
}

impl ThreadStateTracker {
    pub fn state(&self) -> ThreadState {
        match self.inner.load(Ordering::Relaxed) {
            0 => ThreadState::Idle,
            1 => ThreadState::Processing,
            x => panic!("Invalid JobState: {}", x),
        }
    }
}

pub struct JobScheduler {
    handles: Vec<(JoinHandle<()>, ThreadStateTracker)>,
    job_queue: Arc<SegQueue<(JobStateTracker, Box<dyn Job>)>>,
    terminate: Arc<AtomicBool>,
    device: Option<Arc<Device>>,
    queue: Option<Arc<Queue>>,
    condvar: Arc<(Condvar, Mutex<()>)>,
}

impl JobScheduler {
    pub fn init() -> Self {
        Self {
            handles: Vec::new(),
            job_queue: Arc::new(SegQueue::new()),
            terminate: Arc::new(AtomicBool::new(false)),
            device: None,
            queue: None,
            condvar: Arc::new((Condvar::new(), Mutex::new(()))),
        }
    }

    pub fn init_device_queue(device: Arc<Device>, queue: Arc<Queue>) {
        Self::with_lock(|scheduler| {
            scheduler.device = Some(device);
            scheduler.queue = Some(queue);
        })
    }

    fn with_lock<R, F: FnOnce(&mut JobScheduler) -> R>(fun: F) -> R {
        let mut lock = JOB_SCHEDULER.lock();
        fun(&mut lock)
    }

    pub fn spawn_worker() -> Result<()> {
        let (job_queue, terminate, device, queue, condvar) = Self::with_lock(|scheduler| {
            (
                Arc::clone(&scheduler.job_queue),
                Arc::clone(&scheduler.terminate),
                clone_or_panic(&scheduler.device, "expected a initialized job scheduler"),
                clone_or_panic(&scheduler.queue, "expected a initialized job scheduler"),
                Arc::clone(&scheduler.condvar),
            )
        });

        let thread_state_tracker = Arc::new(AtomicU8::new(ThreadState::Idle as u8));
        let t_thread_state_tracker = Arc::clone(&thread_state_tracker);
        let handle = thread::Builder::new()
            .name("JobWorker".to_string())
            .spawn(|| {
                worker_main(
                    device,
                    queue,
                    condvar,
                    job_queue,
                    terminate,
                    t_thread_state_tracker,
                )
            })?;

        Self::with_lock(|scheduler| {
            scheduler.handles.push((
                handle,
                ThreadStateTracker {
                    inner: thread_state_tracker,
                },
            ))
        });

        Ok(())
    }

    pub fn spawn_workers(count: usize) -> Result<()> {
        for _ in 0..count {
            Self::spawn_worker()?;
        }
        Ok(())
    }

    pub fn submit(job: Box<dyn Job>) -> JobStateTracker {
        let state = Arc::new(AtomicU8::new(JobState::Queued as u8));
        let condvar = Arc::new((Condvar::new(), Mutex::new(())));
        let tracker = JobStateTracker { state, condvar };

        let tracker_clone = tracker.clone();

        Self::with_lock(|scheduler| {
            scheduler.job_queue.push((tracker_clone, job));
            if scheduler.handles.len() == 0 {
                warn!("No JobWorkers are running, but a job was submitted!");
            }
            if !scheduler.condvar.0.notify_one() {
                warn!("Condvar did not wake up a JobWorker!");
            }
        });

        tracker
    }

    pub fn thread_states() -> Vec<ThreadState> {
        Self::with_lock(|scheduler| {
            scheduler
                .handles
                .iter()
                .map(|(_, state)| state.state())
                .collect()
        })
    }
}

fn clone_or_panic<T>(v: &Option<Arc<T>>, msg: &str) -> Arc<T> {
    match v.as_ref() {
        Some(x) => Arc::clone(x),
        None => panic!("{}", msg),
    }
}

fn worker_main(
    device: Arc<Device>,
    queue: Arc<Queue>,
    condvar: Arc<(Condvar, Mutex<()>)>,
    job_queue: Arc<SegQueue<(JobStateTracker, Box<dyn Job>)>>,
    terminate: Arc<AtomicBool>,
    thread_state: Arc<AtomicU8>,
) {
    loop {
        condvar.0.wait(&mut condvar.1.lock());
        match job_queue.pop() {
            Some((job_state, mut job)) => {
                thread_state.store(ThreadState::Processing as u8, Ordering::Relaxed);
                job_state
                    .state
                    .store(JobState::Processing as u8, Ordering::Relaxed);
                let start = Instant::now();
                match job.run(&device, &queue) {
                    Ok(()) => {
                        if job.get_freq() != JobFrequency::Frame {
                            // we don't want to flood the logs with jobs that happen every frame
                            info!(
                                "Job {} finished in {:.2} ms",
                                job.type_name(),
                                start.elapsed().as_secs_f64() * 1000.0
                            );
                        }
                        drop(job);
                        job_state
                            .state
                            .store(JobState::Succeeded as u8, Ordering::Relaxed);
                        job_state.condvar.0.notify_all();
                    }
                    Err(e) => {
                        job_state
                            .state
                            .store(JobState::Failed as u8, Ordering::Relaxed);
                        warn!(
                            "Job {} returned an error: {} in {:?} ms",
                            job.type_name(),
                            e,
                            start.elapsed().as_secs_f64() * 1000.0
                        )
                    }
                }
                thread_state.store(ThreadState::Idle as u8, Ordering::Relaxed);
            }
            None => {
                if terminate.load(Ordering::Relaxed) {
                    info!("JobWorker terminating");
                    return;
                }
            }
        }
    }
}

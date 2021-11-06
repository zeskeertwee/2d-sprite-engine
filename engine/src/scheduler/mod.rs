use crate::asset_management::ToUuid;
use anyhow::Result;
use crossbeam::queue::SegQueue;
use lazy_static::lazy_static;
use log::{info, warn};
use parking_lot::Mutex;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Once};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use wgpu::{Device, Queue};

// TODO: Priority levels for jobs?

pub trait Job: Sync + Send + ToUuid {
    fn run(&mut self, device: &Device, queue: &Queue) -> anyhow::Result<()>;
}

lazy_static! {
    static ref JOB_SCHEDULER: Mutex<JobScheduler> = Mutex::new(JobScheduler::init());
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
    inner: Arc<AtomicU8>,
}

impl JobStateTracker {
    pub fn state(&self) -> JobState {
        match self.inner.load(Ordering::Relaxed) {
            0 => JobState::Queued,
            1 => JobState::Processing,
            2 => JobState::Failed,
            3 => JobState::Succeeded,
            x => panic!("Invalid JobState: {}", x),
        }
    }

    pub fn flush(&self) -> Result<(), ()> {
        loop {
            if self.state() == JobState::Succeeded {
                return Ok(());
            }
            if self.state() == JobState::Failed {
                return Err(());
            }
            std::thread::sleep(Duration::from_micros(50));
        }
    }
}

pub struct JobScheduler {
    handles: Vec<JoinHandle<()>>,
    job_queue: Arc<SegQueue<(Arc<AtomicU8>, Box<dyn Job>)>>,
    terminate: Arc<AtomicBool>,
    device: Option<Arc<Device>>,
    queue: Option<Arc<Queue>>,
}

impl JobScheduler {
    pub fn init() -> Self {
        Self {
            handles: Vec::new(),
            job_queue: Arc::new(SegQueue::new()),
            terminate: Arc::new(AtomicBool::new(false)),
            device: None,
            queue: None,
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
        let (job_queue, terminate, device, queue) = Self::with_lock(|scheduler| {
            (
                Arc::clone(&scheduler.job_queue),
                Arc::clone(&scheduler.terminate),
                clone_or_panic(&scheduler.device, "expected a initialized job scheduler"),
                clone_or_panic(&scheduler.queue, "expected a initialized job scheduler"),
            )
        });

        let handle = thread::Builder::new()
            .name("JobWorker".to_string())
            .spawn(|| worker_main(device, queue, job_queue, terminate))?;

        Self::with_lock(|scheduler| scheduler.handles.push(handle));

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
        let state_clone = Arc::clone(&state);

        Self::with_lock(|scheduler| {
            scheduler.job_queue.push((state_clone, job));
            if scheduler.handles.len() == 0 {
                warn!("No JobWorkers are running, but a job was submitted!");
            }
        });

        JobStateTracker { inner: state }
    }

    pub fn flush() {
        loop {
            let jobs = Self::with_lock(|scheduler| scheduler.job_queue.len());
            if jobs == 0 {
                break;
            }

            thread::sleep(Duration::from_micros(50));
        }
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
    job_queue: Arc<SegQueue<(Arc<AtomicU8>, Box<dyn Job>)>>,
    terminate: Arc<AtomicBool>,
) {
    loop {
        match job_queue.pop() {
            Some((job_state, mut job)) => {
                job_state.store(JobState::Processing as u8, Ordering::Relaxed);
                match job.run(&device, &queue) {
                    Ok(()) => {
                        job_state.store(JobState::Succeeded as u8, Ordering::Relaxed);
                        info!("Job {} finished", job.type_name())
                    }
                    Err(e) => {
                        job_state.store(JobState::Failed as u8, Ordering::Relaxed);
                        warn!("Job {} returned an error", job.type_name())
                    }
                }
            }
            None => {
                if terminate.load(Ordering::Relaxed) {
                    info!("JobWorker terminating");
                    return;
                }
                // don't burn the CPU time, but respond quickly to new jobs
                // TODO: optimize this? (wake a single thread with barriers when we get a job?)
                thread::sleep(Duration::from_micros(50));
            }
        }
    }
}

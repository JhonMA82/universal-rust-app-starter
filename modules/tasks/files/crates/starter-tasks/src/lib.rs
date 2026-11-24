#![forbid(unsafe_code)]
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU16, AtomicUsize, Ordering},
        mpsc,
    },
    thread,
};
#[derive(Debug, PartialEq)]
pub enum Error<E> {
    Stale,
    Cancelled,
    WorkerStopped,
    Failed(E),
}
#[derive(Default)]
pub struct Control {
    cancel: AtomicBool,
    progress: AtomicU16,
}
impl Control {
    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Acquire)
    }
    pub fn report(&self, points: u16) {
        self.progress
            .fetch_max(points.min(10_000), Ordering::Relaxed);
    }
}
pub struct Job<T, E> {
    pub id: u64,
    pub origin: u64,
    state: Arc<Control>,
    receiver: mpsc::Receiver<Result<T, E>>,
    finished: bool,
}
impl<T, E> Job<T, E> {
    pub fn cancel(&self) {
        self.state.cancel.store(true, Ordering::Release);
    }
    pub fn progress(&self) -> u16 {
        self.state.progress.load(Ordering::Relaxed)
    }
    pub fn poll(&mut self, current: u64) -> Option<Result<T, Error<E>>> {
        if self.finished {
            return None;
        }
        let result = self.receiver.try_recv();
        if matches!(result, Err(mpsc::TryRecvError::Empty)) {
            return None;
        }
        self.finished = true;
        if self.state.cancelled() {
            return Some(Err(Error::Cancelled));
        }
        if current != self.origin {
            return Some(Err(Error::Stale));
        }
        Some(
            result
                .map_err(|_| Error::WorkerStopped)
                .and_then(|r| r.map_err(Error::Failed)),
        )
    }
}
impl<T, E> Drop for Job<T, E> {
    fn drop(&mut self) {
        self.cancel();
    }
}
pub struct Pool {
    limit: usize,
    active: Arc<AtomicUsize>,
    next: u64,
}
impl Pool {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            active: Arc::new(AtomicUsize::new(0)),
            next: 1,
        }
    }
    pub fn start<T: Send + 'static, E: Send + 'static>(
        &mut self,
        revision: u64,
        run: impl FnOnce(&Control) -> Result<T, E> + Send + 'static,
    ) -> std::io::Result<Job<T, E>> {
        if self.active.load(Ordering::Acquire) >= self.limit {
            return Err(std::io::Error::other("worker pool full"));
        }
        let id = self.next;
        self.next = id
            .checked_add(1)
            .ok_or_else(|| std::io::Error::other("job counter exhausted"))?;
        self.active.fetch_add(1, Ordering::AcqRel);
        struct Slot(Arc<AtomicUsize>);
        impl Drop for Slot {
            fn drop(&mut self) {
                self.0.fetch_sub(1, Ordering::AcqRel);
            }
        }
        let slot = Slot(self.active.clone());
        let state = Arc::new(Control::default());
        let control = state.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        thread::Builder::new()
            .name(format!("starter-job-{id}"))
            .spawn(move || {
                let result = run(&control);
                drop(slot);
                let _ = sender.send(result);
            })?;
        Ok(Job {
            id,
            origin: revision,
            state,
            receiver,
            finished: false,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn complete(job: &mut Job<i32, ()>, revision: u64) -> Result<i32, Error<()>> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(value) = job.poll(revision) {
                return value;
            }
            assert!(std::time::Instant::now() < deadline);
            thread::yield_now();
        }
    }
    #[test]
    fn limits_cancellation_and_stale_outputs() {
        let mut pool = Pool::new(1);
        let (go, wait) = mpsc::channel();
        let mut job = pool
            .start(0, move |cx| {
                wait.recv().unwrap();
                assert!(cx.cancelled());
                Ok(7)
            })
            .unwrap();
        assert!(pool.start(0, |_| Ok::<_, ()>(7)).is_err());
        job.cancel();
        go.send(()).unwrap();
        assert_eq!(complete(&mut job, 0), Err(Error::Cancelled));
        let mut job = pool.start(1, |_| Ok(8)).unwrap();
        assert_eq!(complete(&mut job, 2), Err(Error::Stale));
    }
}

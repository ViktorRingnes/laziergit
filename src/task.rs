use color_eyre::Result;
use color_eyre::eyre::eyre;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

const FRAMES: [char; 4] = ['|', '/', '-', '\\'];

pub struct Task {
    pub label: String,
    rx: Receiver<Result<String>>,
    frame: usize,
}

impl Task {
    pub fn spawn(label: &str, job: fn() -> Result<String>) -> Task {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(job());
        });
        Task {
            label: label.to_string(),
            rx,
            frame: 0,
        }
    }

    pub fn poll(&self) -> Option<Result<String>> {
        match self.rx.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err(eyre!("interrupted"))),
        }
    }

    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

    pub fn spinner(&self) -> char {
        FRAMES[self.frame % FRAMES.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn done() -> Result<String> {
        Ok("done".to_string())
    }

    #[test]
    fn spinner_advances_each_tick() {
        let mut task = Task::spawn("x", done);
        let first = task.spinner();
        task.tick();
        assert_ne!(first, task.spinner());
    }

    #[test]
    fn poll_reports_completion() {
        let task = Task::spawn("x", done);
        let mut result = None;
        for _ in 0..10_000 {
            if let Some(r) = task.poll() {
                result = Some(r);
                break;
            }
            thread::yield_now();
        }
        assert!(matches!(result, Some(Ok(ref s)) if s == "done"));
    }
}

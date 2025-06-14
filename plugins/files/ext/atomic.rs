//! Signals Management for Threaded File Finder

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI16, Ordering};

pub struct Counter(Arc<AtomicI16>);

impl Counter {
    pub fn new() -> Self {
        Self(Arc::new(AtomicI16::new(0)))
    }
    pub fn tick(&self) -> usize {
        match self
            .0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| Some(v + 1))
        {
            Ok(val) => val as usize,
            Err(err) => {
                log::error!("atomic counter update error: {err:?}");
                0
            }
        }
    }
}

impl Clone for Counter {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

pub struct Signal(Arc<AtomicBool>);

impl Signal {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
    pub fn trip(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn is_tripped(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

impl Clone for Signal {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

pub struct SignalManager {
    signals: Vec<Signal>,
}

impl SignalManager {
    pub fn add(&mut self) -> Signal {
        let signal = Signal::new();
        self.signals.push(signal.clone());
        signal
    }
    pub fn trip_all(&mut self) {
        while !self.signals.is_empty() {
            let signal = self.signals.pop().expect("signal missing");
            signal.trip();
        }
    }
}

//! Background scheduler for async MIDI playback.

use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex, Condvar};
use std::thread::{self, JoinHandle};
use std::time::{Duration as StdDuration, Instant};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

use midir::MidiOutputConnection;

/// A scheduled MIDI event.
#[derive(Debug, Clone)]
struct ScheduledEvent {
    time_ms: u64,
    message: Vec<u8>,
}

impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time_ms == other.time_ms
    }
}

impl Eq for ScheduledEvent {}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior
        other.time_ms.cmp(&self.time_ms)
    }
}

/// Messages sent to the scheduler thread.
pub enum SchedulerCommand {
    /// Schedule a MIDI message at a future time (ms from start)
    Schedule { time_ms: u64, message: Vec<u8> },
    /// Update tempo (used to recalculate future events)
    SetTempo(u16),
    /// Stop all notes immediately
    Stop,
    /// Shutdown the scheduler
    Shutdown,
}

/// The scheduler manages a background thread for timed MIDI events.
pub struct Scheduler {
    sender: Sender<SchedulerCommand>,
    thread: Option<JoinHandle<()>>,
    current_time_ms: Arc<Mutex<u64>>,
    idle_signal: Arc<(Mutex<bool>, Condvar)>,
}

impl Scheduler {
    /// Create a new scheduler with the given MIDI connection.
    pub fn new(connection: Arc<Mutex<MidiOutputConnection>>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let current_time_ms = Arc::new(Mutex::new(0u64));
        let idle_signal = Arc::new((Mutex::new(true), Condvar::new()));

        let time_clone = current_time_ms.clone();
        let idle_clone = idle_signal.clone();

        let thread = thread::spawn(move || {
            Self::run(receiver, connection, time_clone, idle_clone);
        });

        Self {
            sender,
            thread: Some(thread),
            current_time_ms,
            idle_signal,
        }
    }

    /// Schedule a MIDI message at the given time offset (ms).
    pub fn schedule(&self, time_ms: u64, message: Vec<u8>) {
        // Mark as busy
        if let Ok(mut idle) = self.idle_signal.0.lock() {
            *idle = false;
        }
        let _ = self.sender.send(SchedulerCommand::Schedule { time_ms, message });
    }

    /// Stop all playing notes.
    pub fn stop(&self) {
        let _ = self.sender.send(SchedulerCommand::Stop);
    }

    /// Wait for all scheduled events to complete.
    pub fn wait(&self) {
        let (lock, cvar) = &*self.idle_signal;
        let mut idle = lock.lock().unwrap();
        while !*idle {
            idle = cvar.wait(idle).unwrap();
        }
    }

    /// Get the current playback time in ms.
    pub fn current_time_ms(&self) -> u64 {
        *self.current_time_ms.lock().unwrap()
    }

    /// Scheduler thread main loop.
    fn run(
        receiver: Receiver<SchedulerCommand>,
        connection: Arc<Mutex<MidiOutputConnection>>,
        current_time_ms: Arc<Mutex<u64>>,
        idle_signal: Arc<(Mutex<bool>, Condvar)>,
    ) {
        let mut queue: BinaryHeap<ScheduledEvent> = BinaryHeap::new();
        let start = Instant::now();

        loop {
            // Update current time
            let now_ms = start.elapsed().as_millis() as u64;
            if let Ok(mut time) = current_time_ms.lock() {
                *time = now_ms;
            }

            // Process any pending commands (non-blocking)
            while let Ok(cmd) = receiver.try_recv() {
                match cmd {
                    SchedulerCommand::Schedule { time_ms, message } => {
                        queue.push(ScheduledEvent { time_ms, message });
                    }
                    SchedulerCommand::Stop => {
                        queue.clear();
                        // Send all notes off on all channels
                        if let Ok(mut conn) = connection.lock() {
                            for ch in 0..16u8 {
                                let _ = conn.send(&[0xB0 | ch, 123, 0]); // All Notes Off
                            }
                        }
                    }
                    SchedulerCommand::SetTempo(_) => {
                        // Tempo changes don't affect already-scheduled events
                    }
                    SchedulerCommand::Shutdown => {
                        return;
                    }
                }
            }

            // Send any events that are due
            while let Some(event) = queue.peek() {
                if event.time_ms <= now_ms {
                    let event = queue.pop().unwrap();
                    if let Ok(mut conn) = connection.lock() {
                        let _ = conn.send(&event.message);
                    }
                } else {
                    break;
                }
            }

            // Update idle status
            if queue.is_empty() {
                let (lock, cvar) = &*idle_signal;
                if let Ok(mut idle) = lock.lock() {
                    *idle = true;
                    cvar.notify_all();
                }
            }

            // Small sleep to prevent busy-waiting
            thread::sleep(StdDuration::from_micros(500));
        }
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        let _ = self.sender.send(SchedulerCommand::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduled_event_ordering() {
        // Earlier events should have higher priority (min-heap)
        let early = ScheduledEvent { time_ms: 100, message: vec![] };
        let late = ScheduledEvent { time_ms: 200, message: vec![] };

        // In a max-heap with reversed ordering, early > late
        assert!(early > late);
    }
}

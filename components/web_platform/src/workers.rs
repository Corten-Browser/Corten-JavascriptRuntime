use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use serde::{Serialize, Deserialize};

/// Message between main thread and worker
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorkerMessage {
    Data(String),  // JSON-serialized data
    Terminate,
}

/// Web Worker
pub struct Worker {
    id: u64,
    sender: mpsc::Sender<WorkerMessage>,
    receiver: Arc<Mutex<mpsc::Receiver<WorkerMessage>>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(script_url: &str) -> Result<Self, String> {
        let (to_worker_tx, to_worker_rx) = mpsc::channel();
        let (from_worker_tx, from_worker_rx) = mpsc::channel();

        let script = script_url.to_string();
        let handle = thread::spawn(move || {
            // Worker thread execution
            Self::worker_thread_main(&script, to_worker_rx, from_worker_tx);
        });

        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        Ok(Self {
            id,
            sender: to_worker_tx,
            receiver: Arc::new(Mutex::new(from_worker_rx)),
            thread_handle: Some(handle),
        })
    }

    fn worker_thread_main(
        _script: &str,
        rx: mpsc::Receiver<WorkerMessage>,
        tx: mpsc::Sender<WorkerMessage>,
    ) {
        // Worker event loop
        loop {
            match rx.recv() {
                Ok(WorkerMessage::Data(data)) => {
                    // Echo back for now (real impl would execute JS)
                    let _ = tx.send(WorkerMessage::Data(data));
                }
                Ok(WorkerMessage::Terminate) | Err(_) => break,
            }
        }
    }

    pub fn post_message(&self, message: &str) -> Result<(), String> {
        self.sender.send(WorkerMessage::Data(message.to_string()))
            .map_err(|e| e.to_string())
    }

    pub fn receive_message(&self) -> Option<String> {
        let rx = self.receiver.lock().ok()?;
        match rx.try_recv() {
            Ok(WorkerMessage::Data(data)) => Some(data),
            _ => None,
        }
    }

    pub fn terminate(&mut self) {
        let _ = self.sender.send(WorkerMessage::Terminate);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    pub fn id(&self) -> u64 { self.id }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.terminate();
    }
}

/// SharedArrayBuffer for shared memory between workers
pub struct SharedArrayBuffer {
    data: Arc<Mutex<Vec<u8>>>,
}

impl SharedArrayBuffer {
    pub fn new(byte_length: usize) -> Self {
        Self {
            data: Arc::new(Mutex::new(vec![0; byte_length])),
        }
    }

    pub fn byte_length(&self) -> usize {
        self.data.lock().unwrap().len()
    }

    pub fn slice(&self, start: usize, end: usize) -> Vec<u8> {
        let data = self.data.lock().unwrap();
        data[start..end.min(data.len())].to_vec()
    }
}

impl Clone for SharedArrayBuffer {
    fn clone(&self) -> Self {
        Self { data: Arc::clone(&self.data) }
    }
}

/// Atomics operations for SharedArrayBuffer
pub struct Atomics;

impl Atomics {
    pub fn load(buffer: &SharedArrayBuffer, index: usize) -> i32 {
        let data = buffer.data.lock().unwrap();
        if index + 4 <= data.len() {
            i32::from_le_bytes(data[index..index+4].try_into().unwrap())
        } else {
            0
        }
    }

    pub fn store(buffer: &SharedArrayBuffer, index: usize, value: i32) {
        let mut data = buffer.data.lock().unwrap();
        if index + 4 <= data.len() {
            data[index..index+4].copy_from_slice(&value.to_le_bytes());
        }
    }

    pub fn add(buffer: &SharedArrayBuffer, index: usize, value: i32) -> i32 {
        let mut data = buffer.data.lock().unwrap();
        if index + 4 <= data.len() {
            let old = i32::from_le_bytes(data[index..index+4].try_into().unwrap());
            let new = old.wrapping_add(value);
            data[index..index+4].copy_from_slice(&new.to_le_bytes());
            old
        } else {
            0
        }
    }

    pub fn compare_exchange(buffer: &SharedArrayBuffer, index: usize, expected: i32, replacement: i32) -> i32 {
        let mut data = buffer.data.lock().unwrap();
        if index + 4 <= data.len() {
            let current = i32::from_le_bytes(data[index..index+4].try_into().unwrap());
            if current == expected {
                data[index..index+4].copy_from_slice(&replacement.to_le_bytes());
            }
            current
        } else {
            0
        }
    }
}

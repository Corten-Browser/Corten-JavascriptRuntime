use web_platform::{Worker, SharedArrayBuffer, Atomics};
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod worker_tests {
    use super::*;

    #[test]
    fn test_worker_creation() {
        let worker = Worker::new("test.js").expect("Should create worker");
        assert!(worker.id() > 0);
    }

    #[test]
    fn test_worker_unique_ids() {
        let worker1 = Worker::new("test1.js").expect("Should create worker 1");
        let worker2 = Worker::new("test2.js").expect("Should create worker 2");
        assert_ne!(worker1.id(), worker2.id());
    }

    #[test]
    fn test_worker_post_message() {
        let worker = Worker::new("test.js").expect("Should create worker");
        let result = worker.post_message("Hello, Worker!");
        assert!(result.is_ok());
    }

    #[test]
    fn test_worker_message_roundtrip() {
        let worker = Worker::new("test.js").expect("Should create worker");
        worker.post_message("test message").expect("Should post message");

        // Give worker time to process
        thread::sleep(Duration::from_millis(50));

        let response = worker.receive_message();
        assert_eq!(response, Some("test message".to_string()));
    }

    #[test]
    fn test_worker_multiple_messages() {
        let worker = Worker::new("test.js").expect("Should create worker");

        worker.post_message("msg1").expect("Should post msg1");
        worker.post_message("msg2").expect("Should post msg2");
        worker.post_message("msg3").expect("Should post msg3");

        thread::sleep(Duration::from_millis(100));

        assert_eq!(worker.receive_message(), Some("msg1".to_string()));
        assert_eq!(worker.receive_message(), Some("msg2".to_string()));
        assert_eq!(worker.receive_message(), Some("msg3".to_string()));
    }

    #[test]
    fn test_worker_no_message_available() {
        let worker = Worker::new("test.js").expect("Should create worker");
        let response = worker.receive_message();
        assert_eq!(response, None);
    }

    #[test]
    fn test_worker_termination() {
        let mut worker = Worker::new("test.js").expect("Should create worker");
        let id = worker.id();
        worker.terminate();
        // After termination, worker should still have its ID
        assert_eq!(worker.id(), id);
    }

    #[test]
    fn test_worker_drop_terminates() {
        let id;
        {
            let worker = Worker::new("test.js").expect("Should create worker");
            id = worker.id();
            worker.post_message("test").expect("Should post");
            // Worker dropped here, should terminate cleanly
        }
        // Just verify no panic occurred
        assert!(id > 0);
    }

    #[test]
    fn test_multiple_workers() {
        let workers: Vec<_> = (0..3)
            .map(|i| Worker::new(&format!("worker{}.js", i)).expect("Should create worker"))
            .collect();

        // All workers should have unique IDs
        let ids: Vec<_> = workers.iter().map(|w| w.id()).collect();
        assert_eq!(ids.len(), 3);
        assert_ne!(ids[0], ids[1]);
        assert_ne!(ids[1], ids[2]);
        assert_ne!(ids[0], ids[2]);
    }
}

#[cfg(test)]
mod shared_array_buffer_tests {
    use super::*;

    #[test]
    fn test_shared_array_buffer_creation() {
        let buffer = SharedArrayBuffer::new(1024);
        assert_eq!(buffer.byte_length(), 1024);
    }

    #[test]
    fn test_shared_array_buffer_zero_length() {
        let buffer = SharedArrayBuffer::new(0);
        assert_eq!(buffer.byte_length(), 0);
    }

    #[test]
    fn test_shared_array_buffer_initialized_to_zero() {
        let buffer = SharedArrayBuffer::new(16);
        let slice = buffer.slice(0, 16);
        assert_eq!(slice, vec![0u8; 16]);
    }

    #[test]
    fn test_shared_array_buffer_slice() {
        let buffer = SharedArrayBuffer::new(100);
        let slice = buffer.slice(10, 20);
        assert_eq!(slice.len(), 10);
        assert_eq!(slice, vec![0u8; 10]);
    }

    #[test]
    fn test_shared_array_buffer_slice_bounds() {
        let buffer = SharedArrayBuffer::new(50);
        // Request beyond length, should clamp
        let slice = buffer.slice(40, 100);
        assert_eq!(slice.len(), 10); // 50 - 40 = 10
    }

    #[test]
    fn test_shared_array_buffer_clone_shares_memory() {
        let buffer1 = SharedArrayBuffer::new(16);
        let buffer2 = buffer1.clone();

        // Both should have same byte length
        assert_eq!(buffer1.byte_length(), buffer2.byte_length());

        // Write to buffer1 via Atomics
        Atomics::store(&buffer1, 0, 42);

        // Should be visible in buffer2
        let value = Atomics::load(&buffer2, 0);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_shared_array_buffer_multiple_clones() {
        let buffer1 = SharedArrayBuffer::new(32);
        let buffer2 = buffer1.clone();
        let buffer3 = buffer2.clone();

        Atomics::store(&buffer1, 0, 100);
        assert_eq!(Atomics::load(&buffer2, 0), 100);
        assert_eq!(Atomics::load(&buffer3, 0), 100);

        Atomics::store(&buffer3, 4, 200);
        assert_eq!(Atomics::load(&buffer1, 4), 200);
        assert_eq!(Atomics::load(&buffer2, 4), 200);
    }
}

#[cfg(test)]
mod atomics_tests {
    use super::*;

    #[test]
    fn test_atomics_load_initial() {
        let buffer = SharedArrayBuffer::new(16);
        let value = Atomics::load(&buffer, 0);
        assert_eq!(value, 0);
    }

    #[test]
    fn test_atomics_store_and_load() {
        let buffer = SharedArrayBuffer::new(16);
        Atomics::store(&buffer, 0, 12345);
        let value = Atomics::load(&buffer, 0);
        assert_eq!(value, 12345);
    }

    #[test]
    fn test_atomics_store_multiple_indices() {
        let buffer = SharedArrayBuffer::new(32);
        Atomics::store(&buffer, 0, 100);
        Atomics::store(&buffer, 4, 200);
        Atomics::store(&buffer, 8, 300);

        assert_eq!(Atomics::load(&buffer, 0), 100);
        assert_eq!(Atomics::load(&buffer, 4), 200);
        assert_eq!(Atomics::load(&buffer, 8), 300);
    }

    #[test]
    fn test_atomics_store_negative() {
        let buffer = SharedArrayBuffer::new(16);
        Atomics::store(&buffer, 0, -42);
        let value = Atomics::load(&buffer, 0);
        assert_eq!(value, -42);
    }

    #[test]
    fn test_atomics_store_max_value() {
        let buffer = SharedArrayBuffer::new(16);
        Atomics::store(&buffer, 0, i32::MAX);
        assert_eq!(Atomics::load(&buffer, 0), i32::MAX);
    }

    #[test]
    fn test_atomics_store_min_value() {
        let buffer = SharedArrayBuffer::new(16);
        Atomics::store(&buffer, 0, i32::MIN);
        assert_eq!(Atomics::load(&buffer, 0), i32::MIN);
    }

    #[test]
    fn test_atomics_load_out_of_bounds() {
        let buffer = SharedArrayBuffer::new(8);
        // Index 8 would need 4 bytes (8-11), but buffer only has 8 bytes
        let value = Atomics::load(&buffer, 8);
        assert_eq!(value, 0); // Returns 0 for out of bounds
    }

    #[test]
    fn test_atomics_store_out_of_bounds() {
        let buffer = SharedArrayBuffer::new(8);
        // Should not panic, just no-op
        Atomics::store(&buffer, 8, 42);
        // Verify nothing was written
        let slice = buffer.slice(0, 8);
        assert_eq!(slice, vec![0u8; 8]);
    }

    #[test]
    fn test_atomics_add() {
        let buffer = SharedArrayBuffer::new(16);
        Atomics::store(&buffer, 0, 10);
        let old = Atomics::add(&buffer, 0, 5);
        assert_eq!(old, 10); // Returns old value
        assert_eq!(Atomics::load(&buffer, 0), 15); // New value is 10 + 5
    }

    #[test]
    fn test_atomics_add_negative() {
        let buffer = SharedArrayBuffer::new(16);
        Atomics::store(&buffer, 0, 100);
        let old = Atomics::add(&buffer, 0, -30);
        assert_eq!(old, 100);
        assert_eq!(Atomics::load(&buffer, 0), 70);
    }

    #[test]
    fn test_atomics_add_wrapping() {
        let buffer = SharedArrayBuffer::new(16);
        Atomics::store(&buffer, 0, i32::MAX);
        let old = Atomics::add(&buffer, 0, 1);
        assert_eq!(old, i32::MAX);
        assert_eq!(Atomics::load(&buffer, 0), i32::MIN); // Wraps around
    }

    #[test]
    fn test_atomics_add_out_of_bounds() {
        let buffer = SharedArrayBuffer::new(8);
        let old = Atomics::add(&buffer, 8, 5);
        assert_eq!(old, 0); // Returns 0 for out of bounds
    }

    #[test]
    fn test_atomics_compare_exchange_success() {
        let buffer = SharedArrayBuffer::new(16);
        Atomics::store(&buffer, 0, 42);
        let old = Atomics::compare_exchange(&buffer, 0, 42, 100);
        assert_eq!(old, 42); // Returns old value
        assert_eq!(Atomics::load(&buffer, 0), 100); // Exchange happened
    }

    #[test]
    fn test_atomics_compare_exchange_failure() {
        let buffer = SharedArrayBuffer::new(16);
        Atomics::store(&buffer, 0, 42);
        let old = Atomics::compare_exchange(&buffer, 0, 99, 100);
        assert_eq!(old, 42); // Returns current value
        assert_eq!(Atomics::load(&buffer, 0), 42); // No exchange
    }

    #[test]
    fn test_atomics_compare_exchange_out_of_bounds() {
        let buffer = SharedArrayBuffer::new(8);
        let old = Atomics::compare_exchange(&buffer, 8, 0, 100);
        assert_eq!(old, 0); // Returns 0 for out of bounds
    }

    #[test]
    fn test_atomics_thread_safety() {
        let buffer = SharedArrayBuffer::new(16);
        Atomics::store(&buffer, 0, 0);

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let buf = buffer.clone();
                thread::spawn(move || {
                    for _ in 0..100 {
                        Atomics::add(&buf, 0, 1);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("Thread should complete");
        }

        // 10 threads * 100 increments = 1000
        assert_eq!(Atomics::load(&buffer, 0), 1000);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_worker_with_shared_memory() {
        // Create shared buffer
        let buffer = SharedArrayBuffer::new(16);
        let buffer_clone = buffer.clone();

        // Create worker
        let worker = Worker::new("test.js").expect("Should create worker");

        // Simulate worker writing to shared memory
        Atomics::store(&buffer_clone, 0, 42);

        // Main thread can read from same memory
        let value = Atomics::load(&buffer, 0);
        assert_eq!(value, 42);

        // Worker communication still works
        worker.post_message("test").expect("Should post");
        thread::sleep(Duration::from_millis(50));
        let msg = worker.receive_message();
        assert_eq!(msg, Some("test".to_string()));
    }

    #[test]
    fn test_multiple_workers_shared_buffer() {
        let buffer = SharedArrayBuffer::new(32);

        let workers: Vec<_> = (0..3)
            .map(|i| {
                let buf = buffer.clone();
                let worker = Worker::new(&format!("worker{}.js", i)).expect("Should create");
                // Each worker writes to different index
                Atomics::store(&buf, i * 4, (i as i32) * 10);
                worker
            })
            .collect();

        // Verify all writes are visible
        assert_eq!(Atomics::load(&buffer, 0), 0);
        assert_eq!(Atomics::load(&buffer, 4), 10);
        assert_eq!(Atomics::load(&buffer, 8), 20);

        // All workers should have unique IDs
        let ids: Vec<_> = workers.iter().map(|w| w.id()).collect();
        let unique_count = ids.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, 3);
    }
}

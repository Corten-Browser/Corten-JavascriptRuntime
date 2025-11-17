use web_platform::{Worker, SharedArrayBuffer, Atomics, DevToolsServer, DebugProtocol, SourceMap, ContentSecurityPolicy};
use web_platform::devtools::{ProtocolMessage, CallFrame, Location, Scope, RemoteObject};
use web_platform::source_maps::{SourceMapping, OriginalPosition, GeneratedPosition};
use web_platform::csp::CspViolation;
use std::thread;
use std::time::Duration;
use serde_json::json;

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

#[cfg(test)]
mod source_map_tests {
    use super::*;

    #[test]
    fn test_source_map_new() {
        let map = SourceMap::new();
        assert_eq!(map.version, 3);
        assert_eq!(map.sources.len(), 0);
        assert_eq!(map.names.len(), 0);
        assert_eq!(map.mappings, "");
    }

    #[test]
    fn test_source_map_default() {
        let map = SourceMap::default();
        assert_eq!(map.version, 3);
    }

    #[test]
    fn test_source_map_from_json_simple() {
        let json = r#"{
            "version": 3,
            "file": "output.js",
            "sources": ["input.js"],
            "names": [],
            "mappings": "AAAA"
        }"#;

        let map = SourceMap::from_json(json).expect("Should parse JSON");
        assert_eq!(map.version, 3);
        assert_eq!(map.file, Some("output.js".to_string()));
        assert_eq!(map.sources, vec!["input.js"]);
        assert_eq!(map.mappings_count(), 1);
    }

    #[test]
    fn test_source_map_from_json_with_names() {
        let json = r#"{
            "version": 3,
            "sources": ["app.ts"],
            "names": ["foo", "bar"],
            "mappings": "AAAA,EAAC"
        }"#;

        let map = SourceMap::from_json(json).expect("Should parse JSON");
        assert_eq!(map.names, vec!["foo", "bar"]);
        assert!(map.mappings_count() >= 1);
    }

    #[test]
    fn test_source_map_from_json_invalid() {
        let json = "not valid json";
        let result = SourceMap::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_source_map_to_json() {
        let mut map = SourceMap::new();
        map.file = Some("output.js".to_string());
        map.sources.push("input.js".to_string());

        let json = map.to_json().expect("Should serialize to JSON");
        assert!(json.contains("\"version\":3"));
        assert!(json.contains("\"output.js\""));
        assert!(json.contains("\"input.js\""));
    }

    #[test]
    fn test_vlq_decode_single_positive() {
        let result = SourceMap::encode_vlq(&[0]);
        assert_eq!(result, "A");

        let result = SourceMap::encode_vlq(&[1]);
        assert_eq!(result, "C");

        let result = SourceMap::encode_vlq(&[15]);
        assert_eq!(result, "e");
    }

    #[test]
    fn test_vlq_decode_single_negative() {
        let result = SourceMap::encode_vlq(&[-1]);
        assert_eq!(result, "D");
    }

    #[test]
    fn test_vlq_decode_multiple() {
        let result = SourceMap::encode_vlq(&[0, 0, 0, 0]);
        assert_eq!(result, "AAAA");
    }

    #[test]
    fn test_vlq_decode_large_number() {
        // Large number requires multiple base64 characters
        let result = SourceMap::encode_vlq(&[100]);
        assert!(result.len() > 1);
    }

    #[test]
    fn test_source_map_original_position_for() {
        let json = r#"{
            "version": 3,
            "sources": ["input.js"],
            "names": [],
            "mappings": "AAAA"
        }"#;

        let map = SourceMap::from_json(json).expect("Should parse JSON");
        let result = map.original_position_for(0, 0);

        assert!(result.is_some());
        let (source, line, col) = result.unwrap();
        assert_eq!(source, "input.js");
        assert_eq!(line, 0);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_source_map_original_position_not_found() {
        let json = r#"{
            "version": 3,
            "sources": ["input.js"],
            "names": [],
            "mappings": "AAAA"
        }"#;

        let map = SourceMap::from_json(json).expect("Should parse JSON");
        // Line 10 doesn't exist in mappings
        let result = map.original_position_for(10, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_source_map_multiple_lines() {
        let json = r#"{
            "version": 3,
            "sources": ["input.js"],
            "names": [],
            "mappings": "AAAA;AACA;AACA"
        }"#;

        let map = SourceMap::from_json(json).expect("Should parse JSON");
        assert_eq!(map.mappings_count(), 3);

        // Check first line
        let pos1 = map.original_position_for(0, 0);
        assert!(pos1.is_some());

        // Check second line
        let pos2 = map.original_position_for(1, 0);
        assert!(pos2.is_some());

        // Check third line
        let pos3 = map.original_position_for(2, 0);
        assert!(pos3.is_some());
    }

    #[test]
    fn test_source_map_multiple_segments() {
        let json = r#"{
            "version": 3,
            "sources": ["input.js"],
            "names": [],
            "mappings": "AAAA,EAAC,GAAE"
        }"#;

        let map = SourceMap::from_json(json).expect("Should parse JSON");
        assert_eq!(map.mappings_count(), 3);
    }

    #[test]
    fn test_source_map_add_mapping() {
        let mut map = SourceMap::new();
        map.sources.push("test.js".to_string());

        let mapping = SourceMapping {
            generated_line: 0,
            generated_column: 0,
            source_index: Some(0),
            original_line: Some(10),
            original_column: Some(5),
            name_index: None,
        };

        map.add_mapping(mapping);
        assert_eq!(map.mappings_count(), 1);
    }

    #[test]
    fn test_source_map_add_source() {
        let mut map = SourceMap::new();
        let idx1 = map.add_source("file1.js".to_string());
        let idx2 = map.add_source("file2.js".to_string());

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(map.sources.len(), 2);
    }

    #[test]
    fn test_source_map_add_name() {
        let mut map = SourceMap::new();
        let idx1 = map.add_name("foo".to_string());
        let idx2 = map.add_name("bar".to_string());

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(map.names.len(), 2);
    }

    #[test]
    fn test_source_map_get_mappings() {
        let json = r#"{
            "version": 3,
            "sources": ["input.js"],
            "names": [],
            "mappings": "AAAA,CAAC"
        }"#;

        let map = SourceMap::from_json(json).expect("Should parse JSON");
        let mappings = map.get_mappings();

        assert_eq!(mappings.len(), 2);
        assert_eq!(mappings[0].generated_line, 0);
        assert_eq!(mappings[0].generated_column, 0);
        assert_eq!(mappings[1].generated_column, 1);
    }

    #[test]
    fn test_source_map_original_position_detailed() {
        let json = r#"{
            "version": 3,
            "sources": ["input.js"],
            "names": ["myFunction"],
            "mappings": "AAAAA"
        }"#;

        let map = SourceMap::from_json(json).expect("Should parse JSON");
        let result = map.original_position_for_detailed(0, 0);

        assert!(result.is_some());
        let pos = result.unwrap();
        assert_eq!(pos.source, "input.js");
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);
        assert_eq!(pos.name, Some("myFunction".to_string()));
    }

    #[test]
    fn test_source_map_generated_position_for() {
        let json = r#"{
            "version": 3,
            "sources": ["input.js"],
            "names": [],
            "mappings": "AAAA"
        }"#;

        let map = SourceMap::from_json(json).expect("Should parse JSON");
        let result = map.generated_position_for("input.js", 0, 0);

        assert!(result.is_some());
        let pos = result.unwrap();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_source_map_generated_position_not_found() {
        let json = r#"{
            "version": 3,
            "sources": ["input.js"],
            "names": [],
            "mappings": "AAAA"
        }"#;

        let map = SourceMap::from_json(json).expect("Should parse JSON");
        // Source doesn't exist
        let result = map.generated_position_for("other.js", 0, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_source_map_regenerate_mappings() {
        let mut map = SourceMap::new();
        map.sources.push("test.js".to_string());

        map.add_mapping(SourceMapping {
            generated_line: 0,
            generated_column: 0,
            source_index: Some(0),
            original_line: Some(0),
            original_column: Some(0),
            name_index: None,
        });

        map.regenerate_mappings();
        assert!(!map.mappings.is_empty());
        assert_eq!(map.mappings, "AAAA");
    }

    #[test]
    fn test_source_map_with_source_content() {
        let json = r#"{
            "version": 3,
            "sources": ["input.js"],
            "sourcesContent": ["console.log('hello');"],
            "names": [],
            "mappings": "AAAA"
        }"#;

        let map = SourceMap::from_json(json).expect("Should parse JSON");
        assert!(map.sources_content.is_some());
        let content = map.sources_content.unwrap();
        assert_eq!(content[0], "console.log('hello');");
    }

    #[test]
    fn test_source_map_empty_mappings() {
        let json = r#"{
            "version": 3,
            "sources": [],
            "names": [],
            "mappings": ""
        }"#;

        let map = SourceMap::from_json(json).expect("Should parse JSON");
        assert_eq!(map.mappings_count(), 0);
    }
}

#[cfg(test)]
mod csp_tests {
    use super::*;

    #[test]
    fn test_csp_new() {
        let csp = ContentSecurityPolicy::new();
        assert!(csp.is_empty());
        assert_eq!(csp.directive_count(), 0);
    }

    #[test]
    fn test_csp_default() {
        let csp = ContentSecurityPolicy::default();
        assert!(csp.is_empty());
    }

    #[test]
    fn test_csp_parse_simple() {
        let header = "default-src 'self'";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        assert!(csp.has_directive("default-src"));
        assert_eq!(csp.directive_count(), 1);
    }

    #[test]
    fn test_csp_parse_multiple_directives() {
        let header = "default-src 'self'; script-src 'unsafe-inline'; style-src https:";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        assert!(csp.has_directive("default-src"));
        assert!(csp.has_directive("script-src"));
        assert!(csp.has_directive("style-src"));
        assert_eq!(csp.directive_count(), 3);
    }

    #[test]
    fn test_csp_parse_multiple_values() {
        let header = "script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.example.com";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        let values = csp.get_directive("script-src").expect("Should have directive");
        assert_eq!(values.len(), 4);
        assert!(values.contains(&"'self'".to_string()));
        assert!(values.contains(&"'unsafe-inline'".to_string()));
        assert!(values.contains(&"'unsafe-eval'".to_string()));
    }

    #[test]
    fn test_csp_parse_case_insensitive() {
        let header = "Default-Src 'self'; SCRIPT-SRC 'none'";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        assert!(csp.has_directive("default-src"));
        assert!(csp.has_directive("script-src"));
    }

    #[test]
    fn test_csp_parse_empty_directive() {
        let header = "upgrade-insecure-requests";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        let values = csp.get_directive("upgrade-insecure-requests").expect("Should have directive");
        assert_eq!(values.len(), 0);
    }

    #[test]
    fn test_csp_from_header() {
        let header = "default-src 'self'";
        let csp = ContentSecurityPolicy::from_header(header).expect("Should parse");
        assert!(csp.has_directive("default-src"));
    }

    #[test]
    fn test_csp_allows_eval_allowed() {
        let header = "script-src 'unsafe-eval'";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");
        assert!(csp.allows_eval());
    }

    #[test]
    fn test_csp_allows_eval_not_allowed() {
        let header = "script-src 'self'";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");
        assert!(!csp.allows_eval());
    }

    #[test]
    fn test_csp_allows_inline_script_allowed() {
        let header = "script-src 'unsafe-inline'";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");
        assert!(csp.allows_inline_script());
    }

    #[test]
    fn test_csp_allows_inline_script_not_allowed() {
        let header = "script-src 'self'";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");
        assert!(!csp.allows_inline_script());
    }

    #[test]
    fn test_csp_allows_source_with_default_src() {
        let header = "default-src 'self' https://cdn.example.com";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        // Falls back to default-src
        assert!(csp.allows_script_source("https://cdn.example.com"));
        assert!(csp.allows_style_source("https://cdn.example.com"));
        assert!(csp.allows_image_source("https://cdn.example.com"));
    }

    #[test]
    fn test_csp_allows_source_specific_directive() {
        let header = "default-src 'none'; script-src https://js.example.com";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        // Uses specific directive, not default
        assert!(csp.allows_script_source("https://js.example.com"));
        assert!(!csp.allows_style_source("https://js.example.com"));
    }

    #[test]
    fn test_csp_allows_source_no_policy() {
        let csp = ContentSecurityPolicy::new();
        // No policy means everything allowed
        assert!(csp.allows_eval());
        assert!(csp.allows_inline_script());
        assert!(csp.allows_script_source("https://any.example.com"));
    }

    #[test]
    fn test_csp_allows_source_wildcard() {
        let header = "script-src *";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");
        assert!(csp.allows_script_source("https://any.example.com"));
    }

    #[test]
    fn test_csp_allows_source_prefix_wildcard() {
        let header = "script-src https://cdn.*";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        assert!(csp.allows_script_source("https://cdn.example.com"));
        assert!(csp.allows_script_source("https://cdn.other.com"));
        assert!(!csp.allows_script_source("https://other.example.com"));
    }

    #[test]
    fn test_csp_allows_source_none() {
        let header = "script-src 'none'";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        assert!(!csp.allows_script_source("https://any.example.com"));
        assert!(!csp.allows_eval());
        assert!(!csp.allows_inline_script());
    }

    #[test]
    fn test_csp_add_directive() {
        let mut csp = ContentSecurityPolicy::new();
        csp.add_directive("script-src", vec!["'self'".to_string(), "https://cdn.example.com".to_string()]);

        assert!(csp.has_directive("script-src"));
        let values = csp.get_directive("script-src").unwrap();
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_csp_add_directive_str() {
        let mut csp = ContentSecurityPolicy::new();
        csp.add_directive_str("script-src", vec!["'self'", "https://cdn.example.com"]);

        assert!(csp.has_directive("script-src"));
    }

    #[test]
    fn test_csp_remove_directive() {
        let header = "default-src 'self'; script-src 'none'";
        let mut csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        let removed = csp.remove_directive("script-src");
        assert!(removed.is_some());
        assert!(!csp.has_directive("script-src"));
    }

    #[test]
    fn test_csp_directive_names() {
        let header = "default-src 'self'; script-src 'none'; style-src https:";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        let names = csp.directive_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"default-src".to_string()));
        assert!(names.contains(&"script-src".to_string()));
        assert!(names.contains(&"style-src".to_string()));
    }

    #[test]
    fn test_csp_to_header() {
        let mut csp = ContentSecurityPolicy::new();
        csp.add_directive_str("default-src", vec!["'self'"]);
        csp.add_directive_str("script-src", vec!["'none'"]);

        let header = csp.to_header();
        assert!(header.contains("default-src 'self'"));
        assert!(header.contains("script-src 'none'"));
        assert!(header.contains("; "));
    }

    #[test]
    fn test_csp_to_header_empty_directive() {
        let mut csp = ContentSecurityPolicy::new();
        csp.add_directive("upgrade-insecure-requests", vec![]);

        let header = csp.to_header();
        assert!(header.contains("upgrade-insecure-requests"));
    }

    #[test]
    fn test_csp_strict() {
        let csp = ContentSecurityPolicy::strict();

        assert!(csp.has_directive("default-src"));
        assert!(csp.has_directive("script-src"));
        assert!(csp.has_directive("object-src"));
        assert!(!csp.allows_eval());
        assert!(!csp.allows_inline_script());
    }

    #[test]
    fn test_csp_permissive() {
        let csp = ContentSecurityPolicy::permissive();

        assert!(csp.allows_eval());
        assert!(csp.allows_inline_script());
        assert!(csp.allows_script_source("https://any.example.com"));
    }

    #[test]
    fn test_csp_report_violation() {
        let csp = ContentSecurityPolicy::new();
        let violation = CspViolation {
            directive: "script-src".to_string(),
            blocked_uri: "https://malicious.com/script.js".to_string(),
            document_uri: "https://example.com/page".to_string(),
            violated_directive: "script-src 'self'".to_string(),
        };

        let report = csp.report_violation(violation);
        assert!(report.contains("malicious.com"));
        assert!(report.contains("script-src"));
    }

    #[test]
    fn test_csp_allows_connect_source() {
        let header = "connect-src https://api.example.com";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        assert!(csp.allows_connect_source("https://api.example.com"));
        assert!(!csp.allows_connect_source("https://other.com"));
    }

    #[test]
    fn test_csp_allows_font_source() {
        let header = "font-src https://fonts.googleapis.com";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        assert!(csp.allows_font_source("https://fonts.googleapis.com"));
    }

    #[test]
    fn test_csp_allows_media_source() {
        let header = "media-src https://media.example.com";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        assert!(csp.allows_media_source("https://media.example.com"));
    }

    #[test]
    fn test_csp_allows_object_source() {
        let header = "object-src 'none'";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        assert!(!csp.allows_object_source("https://any.com"));
    }

    #[test]
    fn test_csp_allows_frame_source() {
        let header = "frame-src https://iframe.example.com";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        assert!(csp.allows_frame_source("https://iframe.example.com"));
    }

    #[test]
    fn test_csp_allows_worker_source() {
        let header = "worker-src 'self'";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        // 'self' only matches exact 'self' keyword in our implementation
        assert!(csp.allows_worker_source("'self'"));
        assert!(!csp.allows_worker_source("https://other-origin.com"));
    }

    #[test]
    fn test_csp_validate_nonce() {
        let header = "script-src 'nonce-abc123'";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        assert!(csp.validate_nonce("script-src", "abc123"));
        assert!(!csp.validate_nonce("script-src", "wrong-nonce"));
    }

    #[test]
    fn test_csp_validate_hash() {
        let header = "script-src 'sha256-abc123hash'";
        let csp = ContentSecurityPolicy::parse(header).expect("Should parse");

        assert!(csp.validate_hash("script-src", "sha256", "abc123hash"));
        assert!(!csp.validate_hash("script-src", "sha256", "wrong-hash"));
    }

    #[test]
    fn test_csp_merge() {
        let mut csp1 = ContentSecurityPolicy::new();
        csp1.add_directive_str("script-src", vec!["'self'", "https://a.com", "https://b.com"]);

        let mut csp2 = ContentSecurityPolicy::new();
        csp2.add_directive_str("script-src", vec!["'self'", "https://b.com", "https://c.com"]);
        csp2.add_directive_str("style-src", vec!["'self'"]);

        csp1.merge(&csp2);

        // Intersection of script-src
        let script_sources = csp1.get_directive("script-src").unwrap();
        assert!(script_sources.contains(&"'self'".to_string()));
        assert!(script_sources.contains(&"https://b.com".to_string()));
        assert!(!script_sources.contains(&"https://a.com".to_string()));
        assert!(!script_sources.contains(&"https://c.com".to_string()));

        // style-src added from csp2
        assert!(csp1.has_directive("style-src"));
    }

    #[test]
    fn test_csp_clone() {
        let header = "default-src 'self'; script-src 'none'";
        let csp1 = ContentSecurityPolicy::parse(header).expect("Should parse");
        let csp2 = csp1.clone();

        assert_eq!(csp1.directive_count(), csp2.directive_count());
        assert!(csp2.has_directive("default-src"));
        assert!(csp2.has_directive("script-src"));
    }
}

#[cfg(test)]
mod devtools_tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = DevToolsServer::new();
        assert!(!server.is_paused());
        assert!(server.breakpoints().is_empty());
        assert!(server.call_stack().is_empty());
    }

    #[test]
    fn test_default_creation() {
        let server = DevToolsServer::default();
        assert!(!server.is_paused());
    }

    #[test]
    fn test_type_alias() {
        let server: DebugProtocol = DebugProtocol::new();
        assert!(!server.is_paused());
    }

    #[test]
    fn test_debugger_enable() {
        let mut server = DevToolsServer::new();
        let msg = ProtocolMessage {
            id: Some(1),
            method: Some("Debugger.enable".to_string()),
            params: None,
            result: None,
            error: None,
        };
        let response = server.handle_message(&msg);
        assert_eq!(response.id, Some(1));
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_set_breakpoint() {
        let mut server = DevToolsServer::new();
        let msg = ProtocolMessage {
            id: Some(2),
            method: Some("Debugger.setBreakpoint".to_string()),
            params: Some(json!({
                "location": {
                    "scriptId": "script_1",
                    "lineNumber": 10,
                    "columnNumber": 5
                }
            })),
            result: None,
            error: None,
        };
        let response = server.handle_message(&msg);
        assert_eq!(response.id, Some(2));
        assert!(response.result.is_some());
        assert_eq!(server.breakpoints().len(), 1);
        let bp = server.breakpoints().get("bp_1").unwrap();
        assert_eq!(bp.script_id, "script_1");
        assert_eq!(bp.line_number, 10);
        assert_eq!(bp.column_number, Some(5));
        assert!(bp.enabled);
    }

    #[test]
    fn test_set_breakpoint_with_condition() {
        let mut server = DevToolsServer::new();
        let msg = ProtocolMessage {
            id: Some(3),
            method: Some("Debugger.setBreakpoint".to_string()),
            params: Some(json!({
                "location": { "scriptId": "script_1", "lineNumber": 15 },
                "condition": "x > 10"
            })),
            result: None,
            error: None,
        };
        server.handle_message(&msg);
        let bp = server.breakpoints().get("bp_1").unwrap();
        assert_eq!(bp.condition, Some("x > 10".to_string()));
    }

    #[test]
    fn test_set_multiple_breakpoints() {
        let mut server = DevToolsServer::new();
        for i in 1..=3 {
            let msg = ProtocolMessage {
                id: Some(i),
                method: Some("Debugger.setBreakpoint".to_string()),
                params: Some(json!({
                    "location": { "scriptId": format!("script_{}", i), "lineNumber": i * 10 }
                })),
                result: None,
                error: None,
            };
            server.handle_message(&msg);
        }
        assert_eq!(server.breakpoints().len(), 3);
    }

    #[test]
    fn test_remove_breakpoint() {
        let mut server = DevToolsServer::new();
        let set_msg = ProtocolMessage {
            id: Some(1),
            method: Some("Debugger.setBreakpoint".to_string()),
            params: Some(json!({ "location": { "scriptId": "script_1", "lineNumber": 10 } })),
            result: None,
            error: None,
        };
        server.handle_message(&set_msg);
        assert_eq!(server.breakpoints().len(), 1);

        let remove_msg = ProtocolMessage {
            id: Some(2),
            method: Some("Debugger.removeBreakpoint".to_string()),
            params: Some(json!({ "breakpointId": "bp_1" })),
            result: None,
            error: None,
        };
        server.handle_message(&remove_msg);
        assert!(server.breakpoints().is_empty());
    }

    #[test]
    fn test_pause_and_resume() {
        let mut server = DevToolsServer::new();
        assert!(!server.is_paused());

        let pause_msg = ProtocolMessage {
            id: Some(1),
            method: Some("Debugger.pause".to_string()),
            params: None,
            result: None,
            error: None,
        };
        server.handle_message(&pause_msg);
        assert!(server.is_paused());

        let resume_msg = ProtocolMessage {
            id: Some(2),
            method: Some("Debugger.resume".to_string()),
            params: None,
            result: None,
            error: None,
        };
        server.handle_message(&resume_msg);
        assert!(!server.is_paused());
    }

    #[test]
    fn test_step_commands() {
        let mut server = DevToolsServer::new();
        let commands = vec!["Debugger.stepOver", "Debugger.stepInto", "Debugger.stepOut"];
        for (i, cmd) in commands.iter().enumerate() {
            let msg = ProtocolMessage {
                id: Some(i as u64),
                method: Some(cmd.to_string()),
                params: None,
                result: None,
                error: None,
            };
            let response = server.handle_message(&msg);
            assert!(response.result.is_some());
            assert!(response.error.is_none());
        }
    }

    #[test]
    fn test_runtime_evaluate_number() {
        let mut server = DevToolsServer::new();
        let msg = ProtocolMessage {
            id: Some(1),
            method: Some("Runtime.evaluate".to_string()),
            params: Some(json!({ "expression": "42.5" })),
            result: None,
            error: None,
        };
        let response = server.handle_message(&msg);
        let result = response.result.unwrap();
        assert_eq!(result["result"]["type"], "number");
        assert_eq!(result["result"]["value"], 42.5);
    }

    #[test]
    fn test_runtime_evaluate_string() {
        let mut server = DevToolsServer::new();
        let msg = ProtocolMessage {
            id: Some(1),
            method: Some("Runtime.evaluate".to_string()),
            params: Some(json!({ "expression": "hello world" })),
            result: None,
            error: None,
        };
        let response = server.handle_message(&msg);
        let result = response.result.unwrap();
        assert_eq!(result["result"]["type"], "string");
        assert_eq!(result["result"]["value"], "hello world");
    }

    #[test]
    fn test_runtime_get_properties() {
        let mut server = DevToolsServer::new();
        let msg = ProtocolMessage {
            id: Some(1),
            method: Some("Runtime.getProperties".to_string()),
            params: Some(json!({ "objectId": "obj_1" })),
            result: None,
            error: None,
        };
        let response = server.handle_message(&msg);
        let result = response.result.unwrap();
        assert!(result["result"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_unknown_method() {
        let mut server = DevToolsServer::new();
        let msg = ProtocolMessage {
            id: Some(1),
            method: Some("Unknown.method".to_string()),
            params: None,
            result: None,
            error: None,
        };
        let response = server.handle_message(&msg);
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
    }

    #[test]
    fn test_add_script() {
        let mut server = DevToolsServer::new();
        let id = server.add_script("console.log('hello');".to_string());
        assert_eq!(id, "script_1");
        assert!(server.get_script(&id).is_some());
    }

    #[test]
    fn test_should_pause_at_breakpoint() {
        let mut server = DevToolsServer::new();
        let msg = ProtocolMessage {
            id: Some(1),
            method: Some("Debugger.setBreakpoint".to_string()),
            params: Some(json!({ "location": { "scriptId": "script_1", "lineNumber": 10 } })),
            result: None,
            error: None,
        };
        server.handle_message(&msg);
        assert!(server.should_pause_at("script_1", 10));
        assert!(!server.should_pause_at("script_1", 11));
        assert!(!server.should_pause_at("script_2", 10));
    }

    #[test]
    fn test_call_stack_operations() {
        let mut server = DevToolsServer::new();
        let frame = CallFrame {
            call_frame_id: "frame_1".to_string(),
            function_name: "main".to_string(),
            location: Location {
                script_id: "script_1".to_string(),
                line_number: 1,
                column_number: 0,
            },
            scope_chain: vec![],
        };
        server.push_call_frame(frame);
        assert_eq!(server.call_stack().len(), 1);

        let popped = server.pop_call_frame().unwrap();
        assert_eq!(popped.function_name, "main");
        assert!(server.call_stack().is_empty());
        assert!(server.pop_call_frame().is_none());
    }

    #[test]
    fn test_next_object_id() {
        let mut server = DevToolsServer::new();
        assert_eq!(server.next_object_id(), "obj_1");
        assert_eq!(server.next_object_id(), "obj_2");
        assert_eq!(server.next_object_id(), "obj_3");
    }

    #[test]
    fn test_remote_object_serialization() {
        let obj = RemoteObject {
            object_type: "object".to_string(),
            value: Some(json!({"key": "value"})),
            description: Some("Object".to_string()),
            object_id: Some("obj_1".to_string()),
        };
        let json_val = serde_json::to_value(&obj).unwrap();
        assert_eq!(json_val["type"], "object");
        assert_eq!(json_val["value"]["key"], "value");
    }

    #[test]
    fn test_call_frame_with_scope_chain() {
        let frame = CallFrame {
            call_frame_id: "frame_1".to_string(),
            function_name: "test".to_string(),
            location: Location {
                script_id: "script_1".to_string(),
                line_number: 10,
                column_number: 5,
            },
            scope_chain: vec![
                Scope {
                    scope_type: "local".to_string(),
                    object: RemoteObject {
                        object_type: "object".to_string(),
                        value: None,
                        description: Some("Local".to_string()),
                        object_id: Some("scope_1".to_string()),
                    },
                },
            ],
        };
        let json_val = serde_json::to_value(&frame).unwrap();
        assert_eq!(json_val["call_frame_id"], "frame_1");
        assert_eq!(json_val["scope_chain"][0]["type"], "local");
    }

    #[test]
    fn test_location_serialization() {
        let loc = Location {
            script_id: "script_1".to_string(),
            line_number: 42,
            column_number: 10,
        };
        let json_val = serde_json::to_value(&loc).unwrap();
        assert_eq!(json_val["script_id"], "script_1");
        assert_eq!(json_val["line_number"], 42);
        assert_eq!(json_val["column_number"], 10);
    }

    #[test]
    fn test_resume_clears_call_stack() {
        let mut server = DevToolsServer::new();
        let frame = CallFrame {
            call_frame_id: "frame_1".to_string(),
            function_name: "test".to_string(),
            location: Location {
                script_id: "script_1".to_string(),
                line_number: 10,
                column_number: 0,
            },
            scope_chain: vec![],
        };
        server.push_call_frame(frame);
        assert_eq!(server.call_stack().len(), 1);

        let msg = ProtocolMessage {
            id: Some(1),
            method: Some("Debugger.resume".to_string()),
            params: None,
            result: None,
            error: None,
        };
        server.handle_message(&msg);
        assert!(server.call_stack().is_empty());
    }
}

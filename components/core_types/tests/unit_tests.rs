//! Integration test runner for unit tests
//! This file makes cargo test discover the unit test modules

#[path = "unit/test_source.rs"]
mod test_source;

#[path = "unit/test_error.rs"]
mod test_error;

#[path = "unit/test_value.rs"]
mod test_value;

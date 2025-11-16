//! Contract tests for async_runtime component
//!
//! These tests verify that the async_runtime component satisfies the
//! contracts defined in contracts/async_runtime.yaml

use async_runtime::{
    EventLoop, ImportEntry, MicroTask, Module, ModuleStatus, Promise, PromiseReaction,
    PromiseState, Task,
};
use core_types::{ErrorKind, JsError, Value};

mod event_loop_contract {
    use super::*;

    #[test]
    fn event_loop_new_returns_self() {
        let event_loop = EventLoop::new();
        // EventLoop::new() returns Self as per contract
        let _ = event_loop;
    }

    #[test]
    fn event_loop_enqueue_task_accepts_task() {
        let mut event_loop = EventLoop::new();
        let task = Task::new(|| Ok(Value::Undefined));
        event_loop.enqueue_task(task);
        // enqueue_task takes Task and returns ()
    }

    #[test]
    fn event_loop_enqueue_microtask_accepts_microtask() {
        let mut event_loop = EventLoop::new();
        let microtask = MicroTask::new(|| Ok(Value::Undefined));
        event_loop.enqueue_microtask(microtask);
        // enqueue_microtask takes MicroTask and returns ()
    }
}

mod promise_contract {
    use super::*;

    #[test]
    fn promise_new_returns_self() {
        let promise = Promise::new();
        // Promise::new() returns Self
        let _ = promise;
    }

    #[test]
    fn promise_has_state_field() {
        let promise = Promise::new();
        let _state: &PromiseState = &promise.state;
        // Promise has state field of type PromiseState
    }

    #[test]
    fn promise_has_reactions_field() {
        let promise = Promise::new();
        let _reactions: &Vec<PromiseReaction> = &promise.reactions;
        // Promise has reactions field of type Vec<PromiseReaction>
    }

    #[test]
    fn promise_has_result_field() {
        let promise = Promise::new();
        let _result: &Option<Value> = &promise.result;
        // Promise has result field of type Option<Value>
    }

    #[test]
    fn promise_resolve_takes_value() {
        let mut promise = Promise::new();
        promise.resolve(Value::Smi(42));
        // resolve takes Value and returns ()
    }

    #[test]
    fn promise_reject_takes_error() {
        let mut promise = Promise::new();
        let error = JsError {
            kind: ErrorKind::TypeError,
            message: "test error".to_string(),
            stack: vec![],
            source_position: None,
        };
        promise.reject(error);
        // reject takes JsError and returns ()
    }

    #[test]
    fn promise_then_returns_promise() {
        let mut promise = Promise::new();
        let chained = promise.then(None, None);
        // then returns Promise
        let _: Promise = chained;
    }
}

mod promise_state_contract {
    use super::*;

    #[test]
    fn promise_state_has_pending_variant() {
        let state = PromiseState::Pending;
        assert!(matches!(state, PromiseState::Pending));
    }

    #[test]
    fn promise_state_has_fulfilled_variant() {
        let state = PromiseState::Fulfilled;
        assert!(matches!(state, PromiseState::Fulfilled));
    }

    #[test]
    fn promise_state_has_rejected_variant() {
        let state = PromiseState::Rejected;
        assert!(matches!(state, PromiseState::Rejected));
    }
}

mod module_contract {
    use super::*;

    #[test]
    fn module_has_source_field() {
        let module = Module::new("export default 42;".to_string());
        let _source: &String = &module.source;
    }

    #[test]
    fn module_has_status_field() {
        let module = Module::new("export default 42;".to_string());
        let _status: &ModuleStatus = &module.status;
    }

    #[test]
    fn module_has_imports_field() {
        let module = Module::new("export default 42;".to_string());
        let _imports: &Vec<ImportEntry> = &module.imports;
    }

    #[test]
    fn module_has_exports_field() {
        let module = Module::new("export default 42;".to_string());
        let _exports: &Vec<async_runtime::ExportEntry> = &module.exports;
    }

    #[test]
    fn module_link_returns_result() {
        let mut module = Module::new("export default 42;".to_string());
        let result: Result<(), JsError> = module.link();
        // link() returns Result<(), JsError>
        let _ = result;
    }

    #[test]
    fn module_evaluate_returns_result_value() {
        let mut module = Module::new("export default 42;".to_string());
        let _ = module.link();
        let result: Result<Value, JsError> = module.evaluate();
        // evaluate() returns Result<Value, JsError>
        let _ = result;
    }
}

mod module_status_contract {
    use super::*;

    #[test]
    fn module_status_has_unlinked_variant() {
        let status = ModuleStatus::Unlinked;
        assert!(matches!(status, ModuleStatus::Unlinked));
    }

    #[test]
    fn module_status_has_linking_variant() {
        let status = ModuleStatus::Linking;
        assert!(matches!(status, ModuleStatus::Linking));
    }

    #[test]
    fn module_status_has_linked_variant() {
        let status = ModuleStatus::Linked;
        assert!(matches!(status, ModuleStatus::Linked));
    }

    #[test]
    fn module_status_has_evaluating_variant() {
        let status = ModuleStatus::Evaluating;
        assert!(matches!(status, ModuleStatus::Evaluating));
    }

    #[test]
    fn module_status_has_evaluated_variant() {
        let status = ModuleStatus::Evaluated;
        assert!(matches!(status, ModuleStatus::Evaluated));
    }

    #[test]
    fn module_status_has_error_variant() {
        let error = JsError {
            kind: ErrorKind::SyntaxError,
            message: "parse error".to_string(),
            stack: vec![],
            source_position: None,
        };
        let status = ModuleStatus::Error(error);
        assert!(matches!(status, ModuleStatus::Error(_)));
    }
}

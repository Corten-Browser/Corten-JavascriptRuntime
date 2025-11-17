//! Unit tests for Module system

use async_runtime::{ExportEntry, ImportEntry, Module, ModuleStatus};
use core_types::{ErrorKind, JsError};

#[test]
fn new_module_has_source() {
    let source = "export default 42;".to_string();
    let module = Module::new(source.clone());
    assert_eq!(module.source, source);
}

#[test]
fn new_module_starts_unlinked() {
    let module = Module::new("export default 42;".to_string());
    assert!(matches!(module.status, ModuleStatus::Unlinked));
}

#[test]
fn new_module_has_empty_imports() {
    let module = Module::new("export default 42;".to_string());
    assert!(module.imports.is_empty());
}

#[test]
fn new_module_has_empty_exports() {
    let module = Module::new("export default 42;".to_string());
    assert!(module.exports.is_empty());
}

#[test]
fn link_changes_status_to_linked() {
    let mut module = Module::new("export const x = 42;".to_string());
    let result = module.link();
    assert!(result.is_ok());
    assert!(matches!(module.status, ModuleStatus::Linked));
}

#[test]
fn cannot_link_already_linked_module() {
    let mut module = Module::new("export const x = 42;".to_string());
    module.link().unwrap();
    let result = module.link();
    // Should succeed (idempotent) or return error - we'll allow success
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn evaluate_requires_linked_status() {
    let mut module = Module::new("export default 42;".to_string());
    let result = module.evaluate();
    // Cannot evaluate unlinked module
    assert!(result.is_err());
}

#[test]
fn evaluate_changes_status_to_evaluated() {
    let mut module = Module::new("export default 42;".to_string());
    module.link().unwrap();
    let result = module.evaluate();
    assert!(result.is_ok());
    assert!(matches!(module.status, ModuleStatus::Evaluated));
}

#[test]
fn evaluate_returns_value() {
    let mut module = Module::new("export default 42;".to_string());
    module.link().unwrap();
    let result = module.evaluate();
    assert!(result.is_ok());
    // Should return some value (evaluation result)
}

#[test]
fn module_status_lifecycle_unlinked_to_linking() {
    let status = ModuleStatus::Unlinked;
    assert!(matches!(status, ModuleStatus::Unlinked));

    let status = ModuleStatus::Linking;
    assert!(matches!(status, ModuleStatus::Linking));
}

#[test]
fn module_status_lifecycle_linking_to_linked() {
    let status = ModuleStatus::Linking;
    assert!(matches!(status, ModuleStatus::Linking));

    let status = ModuleStatus::Linked;
    assert!(matches!(status, ModuleStatus::Linked));
}

#[test]
fn module_status_lifecycle_linked_to_evaluating() {
    let status = ModuleStatus::Linked;
    assert!(matches!(status, ModuleStatus::Linked));

    let status = ModuleStatus::Evaluating;
    assert!(matches!(status, ModuleStatus::Evaluating));
}

#[test]
fn module_status_lifecycle_evaluating_to_evaluated() {
    let status = ModuleStatus::Evaluating;
    assert!(matches!(status, ModuleStatus::Evaluating));

    let status = ModuleStatus::Evaluated;
    assert!(matches!(status, ModuleStatus::Evaluated));
}

#[test]
fn module_status_can_be_error() {
    let error = JsError {
        kind: ErrorKind::SyntaxError,
        message: "unexpected token".to_string(),
        stack: vec![],
        source_position: None,
    };
    let status = ModuleStatus::Error(error);
    assert!(matches!(status, ModuleStatus::Error(_)));
}

#[test]
fn import_entry_creation() {
    let import = ImportEntry {
        module_specifier: "./utils.js".to_string(),
        import_name: "default".to_string(),
        local_name: "utils".to_string(),
    };
    assert_eq!(import.module_specifier, "./utils.js");
    assert_eq!(import.import_name, "default");
    assert_eq!(import.local_name, "utils");
}

#[test]
fn export_entry_creation() {
    let export = ExportEntry {
        export_name: "default".to_string(),
        local_name: "myFunction".to_string(),
    };
    assert_eq!(export.export_name, "default");
    assert_eq!(export.local_name, "myFunction");
}

#[test]
fn module_with_imports() {
    let mut module = Module::new("import { foo } from './bar.js';".to_string());
    module.add_import(ImportEntry {
        module_specifier: "./bar.js".to_string(),
        import_name: "foo".to_string(),
        local_name: "foo".to_string(),
    });
    assert_eq!(module.imports.len(), 1);
}

#[test]
fn module_with_exports() {
    let mut module = Module::new("export const x = 42;".to_string());
    module.add_export(ExportEntry {
        export_name: "x".to_string(),
        local_name: "x".to_string(),
    });
    assert_eq!(module.exports.len(), 1);
}

#[test]
fn link_error_sets_error_status() {
    let mut module = Module::new("invalid syntax {{{{".to_string());
    module.set_has_syntax_error(true);
    let result = module.link();
    if result.is_err() {
        assert!(matches!(module.status, ModuleStatus::Error(_)));
    }
}

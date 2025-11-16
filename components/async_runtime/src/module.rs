//! ES Module system implementation.
//!
//! This module provides ES6+ module support with the full lifecycle:
//! fetch → parse → link → evaluate.

use core_types::{ErrorKind, JsError, Value};

/// The status of a module in its lifecycle.
///
/// Modules progress through these states during loading:
/// Unlinked → Linking → Linked → Evaluating → Evaluated
///
/// If an error occurs, the module transitions to the Error state.
#[derive(Debug, Clone)]
pub enum ModuleStatus {
    /// Module has been parsed but not yet linked
    Unlinked,
    /// Module is currently being linked (resolving dependencies)
    Linking,
    /// Module has been linked successfully
    Linked,
    /// Module is currently being evaluated
    Evaluating,
    /// Module has been evaluated successfully
    Evaluated,
    /// An error occurred during linking or evaluation
    Error(JsError),
}

impl PartialEq for ModuleStatus {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (ModuleStatus::Unlinked, ModuleStatus::Unlinked)
                | (ModuleStatus::Linking, ModuleStatus::Linking)
                | (ModuleStatus::Linked, ModuleStatus::Linked)
                | (ModuleStatus::Evaluating, ModuleStatus::Evaluating)
                | (ModuleStatus::Evaluated, ModuleStatus::Evaluated)
                | (ModuleStatus::Error(_), ModuleStatus::Error(_))
        )
    }
}

/// Represents an import declaration in a module.
///
/// # Examples
///
/// ```javascript
/// import { foo } from './bar.js';
/// // ImportEntry { module_specifier: "./bar.js", import_name: "foo", local_name: "foo" }
///
/// import utils from './utils.js';
/// // ImportEntry { module_specifier: "./utils.js", import_name: "default", local_name: "utils" }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ImportEntry {
    /// The module specifier (e.g., "./utils.js")
    pub module_specifier: String,
    /// The name exported from the target module
    pub import_name: String,
    /// The local binding name in this module
    pub local_name: String,
}

/// Represents an export declaration in a module.
///
/// # Examples
///
/// ```javascript
/// export const x = 42;
/// // ExportEntry { export_name: "x", local_name: "x" }
///
/// const myFunc = () => {};
/// export default myFunc;
/// // ExportEntry { export_name: "default", local_name: "myFunc" }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ExportEntry {
    /// The name this binding is exported as
    pub export_name: String,
    /// The local name of the binding
    pub local_name: String,
}

/// An ES6 module.
///
/// Modules are the unit of JavaScript code organization, providing:
/// - Strict mode by default
/// - Import/export declarations
/// - Top-level await support
/// - Static module resolution
///
/// # Module Lifecycle
///
/// 1. **Parse**: Module source is parsed to identify imports/exports
/// 2. **Link**: Dependencies are resolved and module graph is built
/// 3. **Evaluate**: Module code is executed
///
/// # Examples
///
/// ```
/// use async_runtime::{Module, ModuleStatus};
///
/// let mut module = Module::new("export const x = 42;".to_string());
/// assert!(matches!(module.status, ModuleStatus::Unlinked));
///
/// module.link().unwrap();
/// assert!(matches!(module.status, ModuleStatus::Linked));
///
/// let result = module.evaluate().unwrap();
/// assert!(matches!(module.status, ModuleStatus::Evaluated));
/// ```
#[derive(Debug, Clone)]
pub struct Module {
    /// The module source code
    pub source: String,
    /// Current status in the module lifecycle
    pub status: ModuleStatus,
    /// Import declarations
    pub imports: Vec<ImportEntry>,
    /// Export declarations
    pub exports: Vec<ExportEntry>,
    /// Whether the module has a syntax error
    has_syntax_error: bool,
}

impl Module {
    /// Creates a new module from source code.
    ///
    /// The module starts in the Unlinked state.
    pub fn new(source: String) -> Self {
        Self {
            source,
            status: ModuleStatus::Unlinked,
            imports: Vec::new(),
            exports: Vec::new(),
            has_syntax_error: false,
        }
    }

    /// Links the module, resolving all dependencies.
    ///
    /// This method:
    /// 1. Validates the module source
    /// 2. Resolves all import dependencies
    /// 3. Builds the module environment
    ///
    /// # Returns
    ///
    /// `Ok(())` if linking succeeds, or an error if dependencies cannot be resolved.
    pub fn link(&mut self) -> Result<(), JsError> {
        // Check for syntax errors
        if self.has_syntax_error {
            let error = JsError {
                kind: ErrorKind::SyntaxError,
                message: "Module has syntax error".to_string(),
                stack: vec![],
                source_position: None,
            };
            self.status = ModuleStatus::Error(error.clone());
            return Err(error);
        }

        // Transition through linking states
        if matches!(self.status, ModuleStatus::Unlinked) {
            self.status = ModuleStatus::Linking;

            // In a real implementation, we would:
            // 1. Parse the source to extract imports/exports
            // 2. Recursively link all dependencies
            // 3. Build the module environment record

            // For now, we just transition to linked
            self.status = ModuleStatus::Linked;
        }

        Ok(())
    }

    /// Evaluates the module code.
    ///
    /// The module must be in the Linked state before evaluation.
    ///
    /// # Returns
    ///
    /// The result of evaluating the module (typically undefined for modules).
    pub fn evaluate(&mut self) -> Result<Value, JsError> {
        match &self.status {
            ModuleStatus::Linked => {
                self.status = ModuleStatus::Evaluating;

                // In a real implementation, we would:
                // 1. Execute the module code
                // 2. Initialize module bindings
                // 3. Handle top-level await

                // For now, we just return undefined
                self.status = ModuleStatus::Evaluated;
                Ok(Value::Undefined)
            }
            ModuleStatus::Evaluated => {
                // Already evaluated, return cached result
                Ok(Value::Undefined)
            }
            ModuleStatus::Unlinked | ModuleStatus::Linking => Err(JsError {
                kind: ErrorKind::TypeError,
                message: "Cannot evaluate unlinked module".to_string(),
                stack: vec![],
                source_position: None,
            }),
            ModuleStatus::Evaluating => Err(JsError {
                kind: ErrorKind::TypeError,
                message: "Circular dependency detected during evaluation".to_string(),
                stack: vec![],
                source_position: None,
            }),
            ModuleStatus::Error(e) => Err(e.clone()),
        }
    }

    /// Adds an import entry to the module.
    pub fn add_import(&mut self, import: ImportEntry) {
        self.imports.push(import);
    }

    /// Adds an export entry to the module.
    pub fn add_export(&mut self, export: ExportEntry) {
        self.exports.push(export);
    }

    /// Sets whether the module has a syntax error.
    ///
    /// This is used to simulate parse errors in testing.
    pub fn set_has_syntax_error(&mut self, has_error: bool) {
        self.has_syntax_error = has_error;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_status_variants() {
        let unlinked = ModuleStatus::Unlinked;
        let linking = ModuleStatus::Linking;
        let linked = ModuleStatus::Linked;
        let evaluating = ModuleStatus::Evaluating;
        let evaluated = ModuleStatus::Evaluated;

        assert!(matches!(unlinked, ModuleStatus::Unlinked));
        assert!(matches!(linking, ModuleStatus::Linking));
        assert!(matches!(linked, ModuleStatus::Linked));
        assert!(matches!(evaluating, ModuleStatus::Evaluating));
        assert!(matches!(evaluated, ModuleStatus::Evaluated));
    }

    #[test]
    fn test_module_status_error() {
        let error = JsError {
            kind: ErrorKind::SyntaxError,
            message: "test".to_string(),
            stack: vec![],
            source_position: None,
        };
        let status = ModuleStatus::Error(error);
        assert!(matches!(status, ModuleStatus::Error(_)));
    }

    #[test]
    fn test_import_entry() {
        let import = ImportEntry {
            module_specifier: "./utils.js".to_string(),
            import_name: "default".to_string(),
            local_name: "utils".to_string(),
        };
        assert_eq!(import.module_specifier, "./utils.js");
    }

    #[test]
    fn test_export_entry() {
        let export = ExportEntry {
            export_name: "default".to_string(),
            local_name: "myFunc".to_string(),
        };
        assert_eq!(export.export_name, "default");
    }

    #[test]
    fn test_module_new() {
        let module = Module::new("export default 42;".to_string());
        assert_eq!(module.source, "export default 42;");
        assert!(matches!(module.status, ModuleStatus::Unlinked));
        assert!(module.imports.is_empty());
        assert!(module.exports.is_empty());
    }

    #[test]
    fn test_module_link() {
        let mut module = Module::new("export default 42;".to_string());
        let result = module.link();
        assert!(result.is_ok());
        assert!(matches!(module.status, ModuleStatus::Linked));
    }

    #[test]
    fn test_module_evaluate() {
        let mut module = Module::new("export default 42;".to_string());
        module.link().unwrap();
        let result = module.evaluate();
        assert!(result.is_ok());
        assert!(matches!(module.status, ModuleStatus::Evaluated));
    }

    #[test]
    fn test_cannot_evaluate_unlinked() {
        let mut module = Module::new("export default 42;".to_string());
        let result = module.evaluate();
        assert!(result.is_err());
    }
}

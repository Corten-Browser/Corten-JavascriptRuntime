//! Profiling data collection for JIT compilation decisions
//!
//! This module is placed in core_types to avoid cyclic dependencies
//! between interpreter and jit_compiler.

use crate::Value;

/// Type information for profiling feedback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeInfo {
    /// Numeric value (Smi or Double)
    Number,
    /// Boolean value
    Boolean,
    /// String value
    String,
    /// Object value
    Object,
    /// Undefined value
    Undefined,
    /// Null value
    Null,
}

/// Branch outcome for profiling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchOutcome {
    /// Branch was taken
    Taken,
    /// Branch was not taken
    NotTaken,
}

/// Profiling data for a function or code block
///
/// Collects runtime statistics to inform JIT compilation decisions.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileData {
    /// Number of times this code has been executed
    pub execution_count: u64,
    /// Type feedback collected at various operations
    pub type_feedback: Vec<TypeInfo>,
    /// Branch outcomes at conditional jumps
    pub branch_outcomes: Vec<BranchOutcome>,
}

impl ProfileData {
    /// Create new empty profile data
    pub fn new() -> Self {
        Self {
            execution_count: 0,
            type_feedback: Vec::new(),
            branch_outcomes: Vec::new(),
        }
    }

    /// Record one execution of the code
    pub fn record_execution(&mut self) {
        self.execution_count += 1;
    }

    /// Record type information for an operation
    pub fn record_type(&mut self, info: TypeInfo) {
        self.type_feedback.push(info);
    }

    /// Record type information from a Value
    ///
    /// Automatically determines the TypeInfo based on the Value variant.
    pub fn record_value_type(&mut self, value: &Value) {
        let type_info = match value {
            Value::Smi(_) | Value::Double(_) => TypeInfo::Number,
            Value::Boolean(_) => TypeInfo::Boolean,
            Value::Undefined => TypeInfo::Undefined,
            Value::Null => TypeInfo::Null,
            Value::String(_) => TypeInfo::String,
            Value::HeapObject(_) | Value::NativeObject(_) => TypeInfo::Object,
            Value::NativeFunction(_) => TypeInfo::Object,
        };
        self.type_feedback.push(type_info);
    }

    /// Record a branch outcome
    pub fn record_branch(&mut self, outcome: BranchOutcome) {
        self.branch_outcomes.push(outcome);
    }

    /// Record that a branch was taken
    pub fn record_branch_taken(&mut self) {
        self.branch_outcomes.push(BranchOutcome::Taken);
    }

    /// Record that a branch was not taken
    pub fn record_branch_not_taken(&mut self) {
        self.branch_outcomes.push(BranchOutcome::NotTaken);
    }

    /// Check if code should be compiled to baseline JIT
    ///
    /// Returns true when execution count reaches baseline threshold (~500)
    pub fn should_compile_baseline(&self) -> bool {
        self.execution_count >= 500
    }

    /// Check if code should be compiled to optimizing JIT
    ///
    /// Returns true when execution count reaches optimized threshold (~10,000)
    pub fn should_compile_optimized(&self) -> bool {
        self.execution_count >= 10000
    }

    /// Get the dominant type from feedback (for specialization)
    ///
    /// Returns the most common type if it appears in >90% of samples.
    pub fn dominant_type(&self) -> Option<TypeInfo> {
        if self.type_feedback.is_empty() {
            return None;
        }

        let mut number_count = 0;
        let mut boolean_count = 0;
        let mut string_count = 0;
        let mut object_count = 0;

        for info in &self.type_feedback {
            match info {
                TypeInfo::Number => number_count += 1,
                TypeInfo::Boolean => boolean_count += 1,
                TypeInfo::String => string_count += 1,
                TypeInfo::Object => object_count += 1,
                _ => {}
            }
        }

        let total = self.type_feedback.len();
        let threshold = total * 90 / 100;

        if number_count >= threshold {
            Some(TypeInfo::Number)
        } else if boolean_count >= threshold {
            Some(TypeInfo::Boolean)
        } else if string_count >= threshold {
            Some(TypeInfo::String)
        } else if object_count >= threshold {
            Some(TypeInfo::Object)
        } else {
            None
        }
    }

    /// Check if the code is predominantly branching one way
    ///
    /// Returns Some(true) if mostly taken, Some(false) if mostly not taken,
    /// None if mixed or insufficient data.
    pub fn branch_bias(&self) -> Option<bool> {
        if self.branch_outcomes.is_empty() {
            return None;
        }

        let taken_count = self
            .branch_outcomes
            .iter()
            .filter(|o| **o == BranchOutcome::Taken)
            .count();
        let total = self.branch_outcomes.len();
        let threshold = total * 90 / 100;

        if taken_count >= threshold {
            Some(true)
        } else if (total - taken_count) >= threshold {
            Some(false)
        } else {
            None
        }
    }

    /// Clear all collected profile data
    pub fn clear(&mut self) {
        self.execution_count = 0;
        self.type_feedback.clear();
        self.branch_outcomes.clear();
    }
}

impl Default for ProfileData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_data_new() {
        let profile = ProfileData::new();
        assert_eq!(profile.execution_count, 0);
        assert!(profile.type_feedback.is_empty());
        assert!(profile.branch_outcomes.is_empty());
    }

    #[test]
    fn test_profile_data_default() {
        let profile = ProfileData::default();
        assert_eq!(profile.execution_count, 0);
    }

    #[test]
    fn test_record_execution() {
        let mut profile = ProfileData::new();
        profile.record_execution();
        assert_eq!(profile.execution_count, 1);
        profile.record_execution();
        assert_eq!(profile.execution_count, 2);
    }

    #[test]
    fn test_record_type() {
        let mut profile = ProfileData::new();
        profile.record_type(TypeInfo::Number);
        profile.record_type(TypeInfo::Boolean);
        assert_eq!(profile.type_feedback.len(), 2);
        assert_eq!(profile.type_feedback[0], TypeInfo::Number);
        assert_eq!(profile.type_feedback[1], TypeInfo::Boolean);
    }

    #[test]
    fn test_record_value_type() {
        let mut profile = ProfileData::new();
        profile.record_value_type(&Value::Smi(42));
        profile.record_value_type(&Value::Double(3.14));
        profile.record_value_type(&Value::Boolean(true));
        profile.record_value_type(&Value::Undefined);
        profile.record_value_type(&Value::Null);

        assert_eq!(profile.type_feedback[0], TypeInfo::Number);
        assert_eq!(profile.type_feedback[1], TypeInfo::Number);
        assert_eq!(profile.type_feedback[2], TypeInfo::Boolean);
        assert_eq!(profile.type_feedback[3], TypeInfo::Undefined);
        assert_eq!(profile.type_feedback[4], TypeInfo::Null);
    }

    #[test]
    fn test_record_branch() {
        let mut profile = ProfileData::new();
        profile.record_branch_taken();
        profile.record_branch_not_taken();
        assert_eq!(profile.branch_outcomes.len(), 2);
        assert_eq!(profile.branch_outcomes[0], BranchOutcome::Taken);
        assert_eq!(profile.branch_outcomes[1], BranchOutcome::NotTaken);
    }

    #[test]
    fn test_should_compile_baseline() {
        let mut profile = ProfileData::new();
        assert!(!profile.should_compile_baseline());

        profile.execution_count = 499;
        assert!(!profile.should_compile_baseline());

        profile.execution_count = 500;
        assert!(profile.should_compile_baseline());
    }

    #[test]
    fn test_should_compile_optimized() {
        let mut profile = ProfileData::new();
        assert!(!profile.should_compile_optimized());

        profile.execution_count = 9999;
        assert!(!profile.should_compile_optimized());

        profile.execution_count = 10000;
        assert!(profile.should_compile_optimized());
    }

    #[test]
    fn test_dominant_type_number() {
        let mut profile = ProfileData::new();
        for _ in 0..10 {
            profile.record_type(TypeInfo::Number);
        }
        assert_eq!(profile.dominant_type(), Some(TypeInfo::Number));
    }

    #[test]
    fn test_dominant_type_mixed() {
        let mut profile = ProfileData::new();
        profile.record_type(TypeInfo::Number);
        profile.record_type(TypeInfo::Boolean);
        profile.record_type(TypeInfo::Number);
        profile.record_type(TypeInfo::Boolean);
        assert_eq!(profile.dominant_type(), None);
    }

    #[test]
    fn test_branch_bias_taken() {
        let mut profile = ProfileData::new();
        for _ in 0..10 {
            profile.record_branch_taken();
        }
        assert_eq!(profile.branch_bias(), Some(true));
    }

    #[test]
    fn test_branch_bias_not_taken() {
        let mut profile = ProfileData::new();
        for _ in 0..10 {
            profile.record_branch_not_taken();
        }
        assert_eq!(profile.branch_bias(), Some(false));
    }

    #[test]
    fn test_branch_bias_mixed() {
        let mut profile = ProfileData::new();
        for _ in 0..5 {
            profile.record_branch_taken();
            profile.record_branch_not_taken();
        }
        assert_eq!(profile.branch_bias(), None);
    }

    #[test]
    fn test_clear() {
        let mut profile = ProfileData::new();
        profile.record_execution();
        profile.record_type(TypeInfo::Number);
        profile.record_branch_taken();

        profile.clear();
        assert_eq!(profile.execution_count, 0);
        assert!(profile.type_feedback.is_empty());
        assert!(profile.branch_outcomes.is_empty());
    }
}

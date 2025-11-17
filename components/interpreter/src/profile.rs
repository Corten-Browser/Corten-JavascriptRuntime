//! Profiling data collection for JIT compilation decisions
//!
//! Tracks execution counts, type feedback, and branch outcomes
//! to determine when to compile functions with JIT compilers.

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
    fn test_type_info_equality() {
        assert_eq!(TypeInfo::Number, TypeInfo::Number);
        assert_ne!(TypeInfo::Number, TypeInfo::Boolean);
    }

    #[test]
    fn test_branch_outcome_equality() {
        assert_eq!(BranchOutcome::Taken, BranchOutcome::Taken);
        assert_ne!(BranchOutcome::Taken, BranchOutcome::NotTaken);
    }
}

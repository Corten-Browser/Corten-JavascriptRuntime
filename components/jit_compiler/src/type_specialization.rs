//! Type specialization for JIT compilation
//!
//! This module provides type-based specialization that transforms generic JavaScript
//! operations into type-specific fast paths. It uses runtime type feedback to
//! determine which specializations are profitable and inserts type guards to ensure
//! correctness.
//!
//! # Architecture
//!
//! The type specializer works on the IR representation:
//! 1. Analyzes profiling data to identify monomorphic operations
//! 2. Replaces generic operations with type-specific variants
//! 3. Inserts type guards that validate type assumptions
//! 4. Generates deoptimization points for guard failures
//!
//! # Example
//!
//! ```ignore
//! // Before specialization:
//! Add(None)  // Generic add that handles all JS types
//!
//! // After specialization (with number feedback):
//! TypeGuard(TypeInfo::Number)
//! DeoptPoint(offset)
//! Add(Some(TypeInfo::Number))  // Fast integer/float add
//! ```

use crate::deopt::DeoptReason;
use crate::ir::{IRFunction, IRInstruction, IROpcode};
use core_types::{ProfileData, TypeInfo};
use std::collections::HashMap;

/// Smi (Small Integer) minimum value for 32-bit tagged integers.
/// JavaScript engines typically use 31 bits for Smi to leave one bit for tagging.
pub const SMI_MIN: i64 = -(1_i64 << 30);

/// Smi (Small Integer) maximum value for 32-bit tagged integers.
pub const SMI_MAX: i64 = (1_i64 << 30) - 1;

/// Type guard that validates runtime type assumptions
#[derive(Debug, Clone, PartialEq)]
pub struct TypeGuard {
    /// The expected type for this guard
    pub expected_type: SpecializedType,
    /// Bytecode offset for deoptimization if guard fails
    pub deopt_offset: usize,
    /// Unique identifier for this guard
    pub guard_id: u32,
    /// Number of times this guard has been checked
    pub check_count: u64,
    /// Number of times this guard has failed
    pub failure_count: u64,
}

impl TypeGuard {
    /// Create a new type guard
    pub fn new(expected_type: SpecializedType, deopt_offset: usize, guard_id: u32) -> Self {
        Self {
            expected_type,
            deopt_offset,
            guard_id,
            check_count: 0,
            failure_count: 0,
        }
    }

    /// Record a guard check (success)
    pub fn record_check(&mut self) {
        self.check_count += 1;
    }

    /// Record a guard failure
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
    }

    /// Calculate the failure rate
    pub fn failure_rate(&self) -> f64 {
        if self.check_count == 0 {
            0.0
        } else {
            self.failure_count as f64 / self.check_count as f64
        }
    }

    /// Check if this guard should be considered unstable (too many failures)
    pub fn is_unstable(&self) -> bool {
        self.failure_count >= 3 || (self.check_count >= 100 && self.failure_rate() > 0.1)
    }
}

/// Specialized types for JIT compilation
///
/// These represent the runtime type categories that the JIT can specialize for.
/// More specific than TypeInfo, these include machine-level distinctions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecializedType {
    /// Small integer that fits in a tagged pointer (31 bits)
    Smi,
    /// IEEE 754 double-precision float
    Float64,
    /// JavaScript string
    String,
    /// JavaScript object with known shape
    ObjectWithShape(usize),
    /// Generic object (unknown shape)
    GenericObject,
    /// Boolean value
    Boolean,
    /// Null or undefined
    NullOrUndefined,
    /// Unknown/polymorphic type (cannot specialize)
    Unknown,
}

impl From<TypeInfo> for SpecializedType {
    fn from(info: TypeInfo) -> Self {
        match info {
            TypeInfo::Number => SpecializedType::Float64, // Conservative: use Float64
            TypeInfo::Boolean => SpecializedType::Boolean,
            TypeInfo::String => SpecializedType::String,
            TypeInfo::Object => SpecializedType::GenericObject,
            TypeInfo::Undefined | TypeInfo::Null => SpecializedType::NullOrUndefined,
            TypeInfo::BigInt => SpecializedType::Unknown, // BigInt requires arbitrary precision
        }
    }
}

impl SpecializedType {
    /// Check if this type supports fast integer math
    pub fn supports_smi_math(&self) -> bool {
        matches!(self, SpecializedType::Smi)
    }

    /// Check if this type supports fast float math
    pub fn supports_float_math(&self) -> bool {
        matches!(self, SpecializedType::Smi | SpecializedType::Float64)
    }

    /// Check if this type is a numeric type
    pub fn is_numeric(&self) -> bool {
        matches!(self, SpecializedType::Smi | SpecializedType::Float64)
    }

    /// Convert to TypeInfo for IR representation
    pub fn to_type_info(&self) -> Option<TypeInfo> {
        match self {
            SpecializedType::Smi | SpecializedType::Float64 => Some(TypeInfo::Number),
            SpecializedType::String => Some(TypeInfo::String),
            SpecializedType::Boolean => Some(TypeInfo::Boolean),
            SpecializedType::ObjectWithShape(_) | SpecializedType::GenericObject => {
                Some(TypeInfo::Object)
            }
            SpecializedType::NullOrUndefined => Some(TypeInfo::Undefined),
            SpecializedType::Unknown => None,
        }
    }
}

/// Property access specialization info
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyAccessSpec {
    /// The object shape ID expected
    pub shape_id: usize,
    /// The property offset within the object
    pub property_offset: u32,
    /// The property name being accessed
    pub property_name: String,
    /// Whether this is a load or store
    pub is_load: bool,
}

impl PropertyAccessSpec {
    /// Create a new property access specialization
    pub fn new(shape_id: usize, property_offset: u32, property_name: String, is_load: bool) -> Self {
        Self {
            shape_id,
            property_offset,
            property_name,
            is_load,
        }
    }
}

/// Specialization decision for an operation
#[derive(Debug, Clone, PartialEq)]
pub enum SpecializationDecision {
    /// Specialize for Smi (small integer) operations
    SmiMath,
    /// Specialize for IEEE 754 double operations
    Float64Math,
    /// Specialize for string concatenation
    StringConcat,
    /// Specialize for property access on known shape
    PropertyAccess(PropertyAccessSpec),
    /// Cannot specialize - use generic fallback
    NoSpecialization,
}

/// Configuration for the type specializer
#[derive(Debug, Clone)]
pub struct TypeSpecializerConfig {
    /// Minimum sample count before considering specialization
    pub min_samples: usize,
    /// Threshold for type dominance (0.0-1.0)
    pub dominance_threshold: f64,
    /// Maximum number of guard failures before de-specializing
    pub max_guard_failures: u32,
    /// Enable Smi (small integer) specialization
    pub enable_smi_specialization: bool,
    /// Enable float specialization
    pub enable_float_specialization: bool,
    /// Enable string concatenation specialization
    pub enable_string_specialization: bool,
    /// Enable property access specialization
    pub enable_property_specialization: bool,
}

impl TypeSpecializerConfig {
    /// Create default configuration
    pub fn new() -> Self {
        Self {
            min_samples: 10,
            dominance_threshold: 0.90,
            max_guard_failures: 3,
            enable_smi_specialization: true,
            enable_float_specialization: true,
            enable_string_specialization: true,
            enable_property_specialization: true,
        }
    }

    /// Create conservative configuration (less aggressive specialization)
    pub fn conservative() -> Self {
        Self {
            min_samples: 50,
            dominance_threshold: 0.95,
            max_guard_failures: 2,
            enable_smi_specialization: true,
            enable_float_specialization: true,
            enable_string_specialization: false,
            enable_property_specialization: false,
        }
    }

    /// Create aggressive configuration (more specialization)
    pub fn aggressive() -> Self {
        Self {
            min_samples: 5,
            dominance_threshold: 0.80,
            max_guard_failures: 5,
            enable_smi_specialization: true,
            enable_float_specialization: true,
            enable_string_specialization: true,
            enable_property_specialization: true,
        }
    }
}

impl Default for TypeSpecializerConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about specialization decisions
#[derive(Debug, Clone, Default)]
pub struct SpecializationStats {
    /// Number of operations analyzed
    pub operations_analyzed: u64,
    /// Number of operations specialized
    pub operations_specialized: u64,
    /// Number of Smi specializations
    pub smi_specializations: u64,
    /// Number of float specializations
    pub float_specializations: u64,
    /// Number of string specializations
    pub string_specializations: u64,
    /// Number of property access specializations
    pub property_specializations: u64,
    /// Number of type guards inserted
    pub guards_inserted: u64,
    /// Number of deopt points inserted
    pub deopt_points_inserted: u64,
}

/// Type specializer that transforms generic operations into type-specific variants
///
/// The specializer analyzes type feedback from runtime profiling and transforms
/// IR operations to use specialized fast paths when types are monomorphic.
///
/// # Example
///
/// ```ignore
/// use jit_compiler::type_specialization::{TypeSpecializer, TypeSpecializerConfig};
/// use jit_compiler::ir::IRFunction;
/// use core_types::ProfileData;
///
/// let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());
/// let ir = IRFunction::new();
/// let profile = ProfileData::new();
///
/// let specialized_ir = specializer.specialize(&ir, &profile);
/// ```
pub struct TypeSpecializer {
    /// Configuration for specialization decisions
    config: TypeSpecializerConfig,
    /// Statistics about specialization
    stats: SpecializationStats,
    /// Type guards indexed by guard ID
    guards: HashMap<u32, TypeGuard>,
    /// Next guard ID to assign
    next_guard_id: u32,
    /// Per-instruction type feedback (bytecode offset -> types seen)
    type_feedback_map: HashMap<usize, Vec<SpecializedType>>,
    /// Property shape cache (property name -> shape info)
    shape_cache: HashMap<String, Vec<(usize, u32)>>,
}

impl TypeSpecializer {
    /// Create a new type specializer with the given configuration
    pub fn new(config: TypeSpecializerConfig) -> Self {
        Self {
            config,
            stats: SpecializationStats::default(),
            guards: HashMap::new(),
            next_guard_id: 0,
            type_feedback_map: HashMap::new(),
            shape_cache: HashMap::new(),
        }
    }

    /// Specialize an IR function using profiling data
    ///
    /// This is the main entry point for type specialization. It analyzes the
    /// profile data and transforms operations where type-specific fast paths
    /// are profitable.
    pub fn specialize(&mut self, ir: &IRFunction, profile: &ProfileData) -> IRFunction {
        // Build type feedback map from profile data
        self.build_type_feedback_map(profile);

        let mut specialized = IRFunction::new();
        specialized.constants = ir.constants.clone();
        specialized.register_count = ir.register_count;

        for instruction in &ir.instructions {
            self.stats.operations_analyzed += 1;

            let specialized_instructions = self.specialize_instruction(instruction, profile);
            for spec_inst in specialized_instructions {
                specialized.instructions.push(spec_inst);
            }
        }

        specialized
    }

    /// Build the type feedback map from profile data
    fn build_type_feedback_map(&mut self, profile: &ProfileData) {
        self.type_feedback_map.clear();

        // Convert TypeInfo feedback to SpecializedType
        // In a real implementation, we'd have per-instruction feedback
        // For now, we aggregate all feedback as a general indicator
        for (idx, type_info) in profile.type_feedback.iter().enumerate() {
            let spec_type = self.refine_type_from_feedback(*type_info, profile);
            self.type_feedback_map
                .entry(idx)
                .or_default()
                .push(spec_type);
        }
    }

    /// Refine TypeInfo to SpecializedType based on additional profiling
    fn refine_type_from_feedback(&self, type_info: TypeInfo, profile: &ProfileData) -> SpecializedType {
        match type_info {
            TypeInfo::Number => {
                // Check if we can specialize to Smi based on observed values
                // In a real implementation, we'd track value ranges
                // For now, assume Smi if we have consistent number feedback
                let number_count = profile
                    .type_feedback
                    .iter()
                    .filter(|t| matches!(t, TypeInfo::Number))
                    .count();
                let total = profile.type_feedback.len();

                if total > 0 && number_count == total {
                    // Pure number feedback - could be Smi or Float64
                    // Conservative: use Float64 (Smi requires value range analysis)
                    SpecializedType::Float64
                } else {
                    SpecializedType::Float64
                }
            }
            TypeInfo::String => SpecializedType::String,
            TypeInfo::Boolean => SpecializedType::Boolean,
            TypeInfo::Object => SpecializedType::GenericObject,
            TypeInfo::Undefined => SpecializedType::NullOrUndefined,
            TypeInfo::Null => SpecializedType::NullOrUndefined,
            TypeInfo::BigInt => SpecializedType::Unknown, // BigInt requires arbitrary precision
        }
    }

    /// Specialize a single instruction
    fn specialize_instruction(
        &mut self,
        instruction: &IRInstruction,
        profile: &ProfileData,
    ) -> Vec<IRInstruction> {
        let decision = self.decide_specialization(&instruction.opcode, profile);

        match decision {
            SpecializationDecision::SmiMath => {
                self.specialize_smi_math(instruction)
            }
            SpecializationDecision::Float64Math => {
                self.specialize_float64_math(instruction)
            }
            SpecializationDecision::StringConcat => {
                self.specialize_string_concat(instruction)
            }
            SpecializationDecision::PropertyAccess(spec) => {
                self.specialize_property_access(instruction, spec)
            }
            SpecializationDecision::NoSpecialization => {
                vec![instruction.clone()]
            }
        }
    }

    /// Decide what specialization to apply to an operation
    fn decide_specialization(
        &self,
        opcode: &IROpcode,
        profile: &ProfileData,
    ) -> SpecializationDecision {
        // Check if we have enough samples
        if profile.type_feedback.len() < self.config.min_samples {
            return SpecializationDecision::NoSpecialization;
        }

        match opcode {
            IROpcode::Add(None)
            | IROpcode::Sub(None)
            | IROpcode::Mul(None)
            | IROpcode::Div(None)
            | IROpcode::Mod(None)
            | IROpcode::Exp(None) => {
                self.decide_math_specialization(profile)
            }
            IROpcode::LoadProperty(_) | IROpcode::StoreProperty(_) => {
                // Property specialization requires shape feedback
                // which we don't have in the current profile structure
                SpecializationDecision::NoSpecialization
            }
            _ => SpecializationDecision::NoSpecialization,
        }
    }

    /// Decide specialization for math operations
    fn decide_math_specialization(&self, profile: &ProfileData) -> SpecializationDecision {
        let dominant_type = self.analyze_dominant_type(&profile.type_feedback);

        match dominant_type {
            Some(SpecializedType::Smi) if self.config.enable_smi_specialization => {
                SpecializationDecision::SmiMath
            }
            Some(SpecializedType::Float64) if self.config.enable_float_specialization => {
                SpecializationDecision::Float64Math
            }
            Some(SpecializedType::String) if self.config.enable_string_specialization => {
                SpecializationDecision::StringConcat
            }
            _ => SpecializationDecision::NoSpecialization,
        }
    }

    /// Analyze type feedback to find the dominant type
    fn analyze_dominant_type(&self, feedback: &[TypeInfo]) -> Option<SpecializedType> {
        if feedback.is_empty() {
            return None;
        }

        // Note: We count Number as Float64 conservatively. To distinguish Smi from Float64,
        // we would need value range analysis which tracks actual values seen at runtime.
        let mut float_count = 0;
        let mut string_count = 0;
        let mut boolean_count = 0;
        let mut object_count = 0;
        let mut null_undef_count = 0;

        for info in feedback {
            match info {
                TypeInfo::Number => float_count += 1,
                TypeInfo::String => string_count += 1,
                TypeInfo::Boolean => boolean_count += 1,
                TypeInfo::Object => object_count += 1,
                TypeInfo::Undefined | TypeInfo::Null => null_undef_count += 1,
                TypeInfo::BigInt => {} // BigInt prevents specialization - treat as polymorphic
            }
        }

        let total = feedback.len();
        let threshold = (total as f64 * self.config.dominance_threshold) as usize;

        // Check for dominant type in order of preference
        if float_count >= threshold {
            Some(SpecializedType::Float64)
        } else if string_count >= threshold {
            Some(SpecializedType::String)
        } else if boolean_count >= threshold {
            Some(SpecializedType::Boolean)
        } else if object_count >= threshold {
            Some(SpecializedType::GenericObject)
        } else if null_undef_count >= threshold {
            Some(SpecializedType::NullOrUndefined)
        } else {
            None // Polymorphic - cannot specialize
        }
    }

    /// Specialize an operation for Smi (small integer) math
    fn specialize_smi_math(&mut self, instruction: &IRInstruction) -> Vec<IRInstruction> {
        let offset = instruction.bytecode_offset;
        let mut result = Vec::new();

        // Insert type guard
        let guard_id = self.create_guard(SpecializedType::Smi, offset);
        result.push(IRInstruction::new(
            IROpcode::TypeGuard(TypeInfo::Number),
            offset,
        ));

        // Insert deoptimization point
        result.push(IRInstruction::new(IROpcode::DeoptPoint(guard_id as usize), offset));
        self.stats.deopt_points_inserted += 1;

        // Create specialized operation
        let specialized_op = match &instruction.opcode {
            IROpcode::Add(_) => IROpcode::Add(Some(TypeInfo::Number)),
            IROpcode::Sub(_) => IROpcode::Sub(Some(TypeInfo::Number)),
            IROpcode::Mul(_) => IROpcode::Mul(Some(TypeInfo::Number)),
            IROpcode::Div(_) => IROpcode::Div(Some(TypeInfo::Number)),
            IROpcode::Mod(_) => IROpcode::Mod(Some(TypeInfo::Number)),
            IROpcode::Exp(_) => IROpcode::Exp(Some(TypeInfo::Number)),
            other => other.clone(),
        };

        result.push(IRInstruction::new(specialized_op, offset));

        self.stats.operations_specialized += 1;
        self.stats.smi_specializations += 1;

        result
    }

    /// Specialize an operation for Float64 math
    fn specialize_float64_math(&mut self, instruction: &IRInstruction) -> Vec<IRInstruction> {
        let offset = instruction.bytecode_offset;
        let mut result = Vec::new();

        // Insert type guard
        let guard_id = self.create_guard(SpecializedType::Float64, offset);
        result.push(IRInstruction::new(
            IROpcode::TypeGuard(TypeInfo::Number),
            offset,
        ));

        // Insert deoptimization point
        result.push(IRInstruction::new(IROpcode::DeoptPoint(guard_id as usize), offset));
        self.stats.deopt_points_inserted += 1;

        // Create specialized operation
        let specialized_op = match &instruction.opcode {
            IROpcode::Add(_) => IROpcode::Add(Some(TypeInfo::Number)),
            IROpcode::Sub(_) => IROpcode::Sub(Some(TypeInfo::Number)),
            IROpcode::Mul(_) => IROpcode::Mul(Some(TypeInfo::Number)),
            IROpcode::Div(_) => IROpcode::Div(Some(TypeInfo::Number)),
            IROpcode::Mod(_) => IROpcode::Mod(Some(TypeInfo::Number)),
            IROpcode::Exp(_) => IROpcode::Exp(Some(TypeInfo::Number)),
            other => other.clone(),
        };

        result.push(IRInstruction::new(specialized_op, offset));

        self.stats.operations_specialized += 1;
        self.stats.float_specializations += 1;

        result
    }

    /// Specialize an operation for string concatenation
    fn specialize_string_concat(&mut self, instruction: &IRInstruction) -> Vec<IRInstruction> {
        let offset = instruction.bytecode_offset;
        let mut result = Vec::new();

        // Only Add can be specialized for string concatenation
        if !matches!(instruction.opcode, IROpcode::Add(_)) {
            return vec![instruction.clone()];
        }

        // Insert type guard for string
        let guard_id = self.create_guard(SpecializedType::String, offset);
        result.push(IRInstruction::new(
            IROpcode::TypeGuard(TypeInfo::String),
            offset,
        ));

        // Insert deoptimization point
        result.push(IRInstruction::new(IROpcode::DeoptPoint(guard_id as usize), offset));
        self.stats.deopt_points_inserted += 1;

        // Create specialized string concatenation
        result.push(IRInstruction::new(
            IROpcode::Add(Some(TypeInfo::String)),
            offset,
        ));

        self.stats.operations_specialized += 1;
        self.stats.string_specializations += 1;

        result
    }

    /// Specialize property access for known object shape
    fn specialize_property_access(
        &mut self,
        instruction: &IRInstruction,
        spec: PropertyAccessSpec,
    ) -> Vec<IRInstruction> {
        let offset = instruction.bytecode_offset;
        let mut result = Vec::new();

        // Insert shape guard
        let guard_id = self.create_guard(SpecializedType::ObjectWithShape(spec.shape_id), offset);
        result.push(IRInstruction::new(
            IROpcode::TypeGuard(TypeInfo::Object),
            offset,
        ));

        // Insert deoptimization point
        result.push(IRInstruction::new(IROpcode::DeoptPoint(guard_id as usize), offset));
        self.stats.deopt_points_inserted += 1;

        // Keep original property access (would be replaced with direct offset access
        // in a full implementation)
        result.push(instruction.clone());

        self.stats.operations_specialized += 1;
        self.stats.property_specializations += 1;

        result
    }

    /// Create a new type guard and return its ID
    fn create_guard(&mut self, expected_type: SpecializedType, deopt_offset: usize) -> u32 {
        let guard_id = self.next_guard_id;
        self.next_guard_id += 1;

        let guard = TypeGuard::new(expected_type, deopt_offset, guard_id);
        self.guards.insert(guard_id, guard);
        self.stats.guards_inserted += 1;

        guard_id
    }

    /// Record a guard check result
    pub fn record_guard_check(&mut self, guard_id: u32, success: bool) {
        if let Some(guard) = self.guards.get_mut(&guard_id) {
            guard.record_check();
            if !success {
                guard.record_failure();
            }
        }
    }

    /// Check if a guard should trigger deoptimization
    pub fn should_deoptimize(&self, guard_id: u32) -> Option<DeoptReason> {
        self.guards.get(&guard_id).and_then(|guard| {
            if guard.is_unstable() {
                Some(DeoptReason::TypeGuardFailure)
            } else {
                None
            }
        })
    }

    /// Get specialization statistics
    pub fn stats(&self) -> &SpecializationStats {
        &self.stats
    }

    /// Reset specialization statistics
    pub fn reset_stats(&mut self) {
        self.stats = SpecializationStats::default();
    }

    /// Get a guard by ID
    pub fn get_guard(&self, guard_id: u32) -> Option<&TypeGuard> {
        self.guards.get(&guard_id)
    }

    /// Get all guards
    pub fn guards(&self) -> &HashMap<u32, TypeGuard> {
        &self.guards
    }

    /// Check if a value is a valid Smi
    pub fn is_smi(value: i64) -> bool {
        value >= SMI_MIN && value <= SMI_MAX
    }

    /// Check if two Smi values can be added without overflow
    pub fn smi_add_will_overflow(a: i32, b: i32) -> bool {
        let result = (a as i64) + (b as i64);
        result < SMI_MIN || result > SMI_MAX
    }

    /// Check if two Smi values can be multiplied without overflow
    pub fn smi_mul_will_overflow(a: i32, b: i32) -> bool {
        let result = (a as i64) * (b as i64);
        result < SMI_MIN || result > SMI_MAX
    }

    /// Record shape feedback for property access
    pub fn record_shape_feedback(&mut self, property_name: &str, shape_id: usize, offset: u32) {
        self.shape_cache
            .entry(property_name.to_string())
            .or_default()
            .push((shape_id, offset));
    }

    /// Get the dominant shape for a property
    pub fn get_dominant_shape(&self, property_name: &str) -> Option<(usize, u32)> {
        let entries = self.shape_cache.get(property_name)?;
        if entries.is_empty() {
            return None;
        }

        // Count occurrences of each shape
        let mut shape_counts: HashMap<usize, (u32, usize)> = HashMap::new();
        for (shape_id, offset) in entries {
            shape_counts
                .entry(*shape_id)
                .and_modify(|(_, count)| *count += 1)
                .or_insert((*offset, 1));
        }

        // Find the most common shape
        let total = entries.len();
        let threshold = (total as f64 * self.config.dominance_threshold) as usize;

        shape_counts
            .iter()
            .find(|(_, (_, count))| *count >= threshold)
            .map(|(shape_id, (offset, _))| (*shape_id, *offset))
    }
}

impl Default for TypeSpecializer {
    fn default() -> Self {
        Self::new(TypeSpecializerConfig::default())
    }
}

/// Deoptimization trigger information
#[derive(Debug, Clone, PartialEq)]
pub struct DeoptTrigger {
    /// The guard that failed
    pub guard_id: u32,
    /// Reason for deoptimization
    pub reason: DeoptReason,
    /// Bytecode offset to resume at
    pub resume_offset: usize,
    /// The actual type encountered
    pub actual_type: SpecializedType,
}

impl DeoptTrigger {
    /// Create a new deopt trigger
    pub fn new(
        guard_id: u32,
        reason: DeoptReason,
        resume_offset: usize,
        actual_type: SpecializedType,
    ) -> Self {
        Self {
            guard_id,
            reason,
            resume_offset,
            actual_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode_system::{BytecodeChunk, Opcode};

    fn create_number_profile(count: usize) -> ProfileData {
        let mut profile = ProfileData::new();
        for _ in 0..count {
            profile.record_type(TypeInfo::Number);
        }
        profile
    }

    fn create_string_profile(count: usize) -> ProfileData {
        let mut profile = ProfileData::new();
        for _ in 0..count {
            profile.record_type(TypeInfo::String);
        }
        profile
    }

    fn create_mixed_profile() -> ProfileData {
        let mut profile = ProfileData::new();
        for _ in 0..5 {
            profile.record_type(TypeInfo::Number);
            profile.record_type(TypeInfo::String);
        }
        profile
    }

    #[test]
    fn test_type_specializer_new() {
        let specializer = TypeSpecializer::new(TypeSpecializerConfig::default());
        assert_eq!(specializer.stats().operations_analyzed, 0);
        assert_eq!(specializer.stats().operations_specialized, 0);
    }

    #[test]
    fn test_type_specializer_default() {
        let specializer = TypeSpecializer::default();
        assert_eq!(specializer.stats().operations_analyzed, 0);
    }

    #[test]
    fn test_config_variations() {
        let default_config = TypeSpecializerConfig::new();
        assert_eq!(default_config.min_samples, 10);
        assert_eq!(default_config.dominance_threshold, 0.90);

        let conservative = TypeSpecializerConfig::conservative();
        assert_eq!(conservative.min_samples, 50);
        assert_eq!(conservative.dominance_threshold, 0.95);

        let aggressive = TypeSpecializerConfig::aggressive();
        assert_eq!(aggressive.min_samples, 5);
        assert_eq!(aggressive.dominance_threshold, 0.80);
    }

    #[test]
    fn test_specialize_add_with_number_feedback() {
        let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());
        let profile = create_number_profile(20);

        let mut ir = IRFunction::new();
        ir.emit(IROpcode::Add(None), 0);

        let specialized = specializer.specialize(&ir, &profile);

        // Should have: TypeGuard, DeoptPoint, specialized Add
        assert_eq!(specialized.instruction_count(), 3);
        assert!(matches!(
            specialized.instructions[0].opcode,
            IROpcode::TypeGuard(_)
        ));
        assert!(matches!(
            specialized.instructions[1].opcode,
            IROpcode::DeoptPoint(_)
        ));
        assert!(matches!(
            specialized.instructions[2].opcode,
            IROpcode::Add(Some(TypeInfo::Number))
        ));
    }

    #[test]
    fn test_specialize_sub_with_number_feedback() {
        let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());
        let profile = create_number_profile(20);

        let mut ir = IRFunction::new();
        ir.emit(IROpcode::Sub(None), 0);

        let specialized = specializer.specialize(&ir, &profile);

        assert_eq!(specialized.instruction_count(), 3);
        assert!(matches!(
            specialized.instructions[2].opcode,
            IROpcode::Sub(Some(TypeInfo::Number))
        ));
    }

    #[test]
    fn test_specialize_mul_with_number_feedback() {
        let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());
        let profile = create_number_profile(20);

        let mut ir = IRFunction::new();
        ir.emit(IROpcode::Mul(None), 0);

        let specialized = specializer.specialize(&ir, &profile);

        assert_eq!(specialized.instruction_count(), 3);
        assert!(matches!(
            specialized.instructions[2].opcode,
            IROpcode::Mul(Some(TypeInfo::Number))
        ));
    }

    #[test]
    fn test_specialize_add_with_string_feedback() {
        let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());
        let profile = create_string_profile(20);

        let mut ir = IRFunction::new();
        ir.emit(IROpcode::Add(None), 0);

        let specialized = specializer.specialize(&ir, &profile);

        // Should have: TypeGuard(String), DeoptPoint, specialized Add(String)
        assert_eq!(specialized.instruction_count(), 3);
        assert!(matches!(
            specialized.instructions[2].opcode,
            IROpcode::Add(Some(TypeInfo::String))
        ));
    }

    #[test]
    fn test_no_specialization_with_mixed_feedback() {
        let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());
        let profile = create_mixed_profile();

        let mut ir = IRFunction::new();
        ir.emit(IROpcode::Add(None), 0);

        let specialized = specializer.specialize(&ir, &profile);

        // Should remain generic
        assert_eq!(specialized.instruction_count(), 1);
        assert!(matches!(
            specialized.instructions[0].opcode,
            IROpcode::Add(None)
        ));
    }

    #[test]
    fn test_no_specialization_with_insufficient_samples() {
        let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());
        let profile = create_number_profile(5); // Less than min_samples (10)

        let mut ir = IRFunction::new();
        ir.emit(IROpcode::Add(None), 0);

        let specialized = specializer.specialize(&ir, &profile);

        // Should remain generic due to insufficient samples
        assert_eq!(specialized.instruction_count(), 1);
        assert!(matches!(
            specialized.instructions[0].opcode,
            IROpcode::Add(None)
        ));
    }

    #[test]
    fn test_type_guard_creation() {
        let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());
        let profile = create_number_profile(20);

        let mut ir = IRFunction::new();
        ir.emit(IROpcode::Add(None), 0);

        let _ = specializer.specialize(&ir, &profile);

        assert_eq!(specializer.guards().len(), 1);
        let guard = specializer.get_guard(0).unwrap();
        assert_eq!(guard.guard_id, 0);
        assert_eq!(guard.expected_type, SpecializedType::Float64);
    }

    #[test]
    fn test_guard_check_recording() {
        let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());
        let profile = create_number_profile(20);

        let mut ir = IRFunction::new();
        ir.emit(IROpcode::Add(None), 0);

        let _ = specializer.specialize(&ir, &profile);

        // Record some checks
        specializer.record_guard_check(0, true);
        specializer.record_guard_check(0, true);
        specializer.record_guard_check(0, false);

        let guard = specializer.get_guard(0).unwrap();
        assert_eq!(guard.check_count, 3);
        assert_eq!(guard.failure_count, 1);
    }

    #[test]
    fn test_guard_stability() {
        let mut guard = TypeGuard::new(SpecializedType::Float64, 0, 0);

        // Initially stable
        assert!(!guard.is_unstable());

        // Record failures
        for _ in 0..3 {
            guard.record_check();
            guard.record_failure();
        }

        // Should be unstable after 3 failures
        assert!(guard.is_unstable());
    }

    #[test]
    fn test_guard_failure_rate() {
        let mut guard = TypeGuard::new(SpecializedType::Float64, 0, 0);

        guard.check_count = 100;
        guard.failure_count = 10;

        assert!((guard.failure_rate() - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_smi_range_checking() {
        assert!(TypeSpecializer::is_smi(0));
        assert!(TypeSpecializer::is_smi(SMI_MAX));
        assert!(TypeSpecializer::is_smi(SMI_MIN));
        assert!(!TypeSpecializer::is_smi(SMI_MAX + 1));
        assert!(!TypeSpecializer::is_smi(SMI_MIN - 1));
    }

    #[test]
    fn test_smi_overflow_detection() {
        // No overflow
        assert!(!TypeSpecializer::smi_add_will_overflow(100, 200));

        // Overflow
        assert!(TypeSpecializer::smi_add_will_overflow(i32::MAX / 2, i32::MAX / 2));

        // Multiplication overflow
        assert!(TypeSpecializer::smi_mul_will_overflow(100_000, 100_000));
    }

    #[test]
    fn test_specialized_type_conversion() {
        assert_eq!(SpecializedType::from(TypeInfo::Number), SpecializedType::Float64);
        assert_eq!(SpecializedType::from(TypeInfo::String), SpecializedType::String);
        assert_eq!(SpecializedType::from(TypeInfo::Boolean), SpecializedType::Boolean);
        assert_eq!(SpecializedType::from(TypeInfo::Object), SpecializedType::GenericObject);
    }

    #[test]
    fn test_specialized_type_properties() {
        assert!(SpecializedType::Smi.supports_smi_math());
        assert!(!SpecializedType::Float64.supports_smi_math());

        assert!(SpecializedType::Smi.supports_float_math());
        assert!(SpecializedType::Float64.supports_float_math());
        assert!(!SpecializedType::String.supports_float_math());

        assert!(SpecializedType::Smi.is_numeric());
        assert!(SpecializedType::Float64.is_numeric());
        assert!(!SpecializedType::String.is_numeric());
    }

    #[test]
    fn test_shape_feedback_recording() {
        let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());

        // Record shape feedback for "x" property
        for _ in 0..20 {
            specializer.record_shape_feedback("x", 1, 0);
        }

        let dominant = specializer.get_dominant_shape("x");
        assert_eq!(dominant, Some((1, 0)));
    }

    #[test]
    fn test_shape_feedback_polymorphic() {
        let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());

        // Record mixed shape feedback
        for i in 0..10 {
            specializer.record_shape_feedback("x", i % 5, i as u32);
        }

        // No dominant shape
        let dominant = specializer.get_dominant_shape("x");
        assert_eq!(dominant, None);
    }

    #[test]
    fn test_stats_tracking() {
        let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());
        let profile = create_number_profile(20);

        let mut ir = IRFunction::new();
        ir.emit(IROpcode::Add(None), 0);
        ir.emit(IROpcode::Sub(None), 1);
        ir.emit(IROpcode::Mul(None), 2);

        let _ = specializer.specialize(&ir, &profile);

        let stats = specializer.stats();
        assert_eq!(stats.operations_analyzed, 3);
        assert_eq!(stats.operations_specialized, 3);
        assert_eq!(stats.float_specializations, 3);
        assert_eq!(stats.guards_inserted, 3);
        assert_eq!(stats.deopt_points_inserted, 3);
    }

    #[test]
    fn test_reset_stats() {
        let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());
        let profile = create_number_profile(20);

        let mut ir = IRFunction::new();
        ir.emit(IROpcode::Add(None), 0);

        let _ = specializer.specialize(&ir, &profile);

        assert!(specializer.stats().operations_analyzed > 0);

        specializer.reset_stats();

        assert_eq!(specializer.stats().operations_analyzed, 0);
        assert_eq!(specializer.stats().operations_specialized, 0);
    }

    #[test]
    fn test_deopt_trigger_creation() {
        let trigger = DeoptTrigger::new(
            0,
            DeoptReason::TypeGuardFailure,
            100,
            SpecializedType::String,
        );

        assert_eq!(trigger.guard_id, 0);
        assert_eq!(trigger.reason, DeoptReason::TypeGuardFailure);
        assert_eq!(trigger.resume_offset, 100);
        assert_eq!(trigger.actual_type, SpecializedType::String);
    }

    #[test]
    fn test_property_access_spec() {
        let spec = PropertyAccessSpec::new(1, 8, "foo".to_string(), true);

        assert_eq!(spec.shape_id, 1);
        assert_eq!(spec.property_offset, 8);
        assert_eq!(spec.property_name, "foo");
        assert!(spec.is_load);
    }

    #[test]
    fn test_should_deoptimize() {
        let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());
        let profile = create_number_profile(20);

        let mut ir = IRFunction::new();
        ir.emit(IROpcode::Add(None), 0);

        let _ = specializer.specialize(&ir, &profile);

        // Initially no deopt needed
        assert!(specializer.should_deoptimize(0).is_none());

        // Record failures
        for _ in 0..3 {
            specializer.record_guard_check(0, false);
        }

        // Now should deoptimize
        assert!(specializer.should_deoptimize(0).is_some());
    }

    #[test]
    fn test_non_specializable_operations() {
        let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());
        let profile = create_number_profile(20);

        let mut ir = IRFunction::new();
        ir.emit(IROpcode::Return, 0);
        ir.emit(IROpcode::LoadConst(0), 1);

        let specialized = specializer.specialize(&ir, &profile);

        // These operations should not be specialized
        assert_eq!(specialized.instruction_count(), 2);
        assert!(matches!(
            specialized.instructions[0].opcode,
            IROpcode::Return
        ));
    }

    #[test]
    fn test_from_bytecode_and_specialize() {
        let mut chunk = BytecodeChunk::new();
        let idx1 = chunk.add_constant(bytecode_system::Value::Number(10.0));
        let idx2 = chunk.add_constant(bytecode_system::Value::Number(20.0));
        chunk.emit(Opcode::LoadConstant(idx1));
        chunk.emit(Opcode::LoadConstant(idx2));
        chunk.emit(Opcode::Add);
        chunk.emit(Opcode::Return);

        let ir = IRFunction::from_bytecode(&chunk);

        let mut specializer = TypeSpecializer::new(TypeSpecializerConfig::default());
        let profile = create_number_profile(20);

        let specialized = specializer.specialize(&ir, &profile);

        // The Add operation should be specialized
        let add_count = specialized
            .instructions
            .iter()
            .filter(|i| matches!(i.opcode, IROpcode::Add(Some(TypeInfo::Number))))
            .count();
        assert_eq!(add_count, 1);
    }

    #[test]
    fn test_specialized_type_to_type_info() {
        assert_eq!(SpecializedType::Smi.to_type_info(), Some(TypeInfo::Number));
        assert_eq!(SpecializedType::Float64.to_type_info(), Some(TypeInfo::Number));
        assert_eq!(SpecializedType::String.to_type_info(), Some(TypeInfo::String));
        assert_eq!(SpecializedType::Boolean.to_type_info(), Some(TypeInfo::Boolean));
        assert_eq!(SpecializedType::GenericObject.to_type_info(), Some(TypeInfo::Object));
        assert_eq!(SpecializedType::Unknown.to_type_info(), None);
    }
}

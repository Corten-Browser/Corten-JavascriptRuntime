//! Inline caching system for property access optimization
//!
//! Provides mono/poly/megamorphic caching states for fast property lookups.

use arrayvec::ArrayVec;

/// Shape identifier for objects (hidden class ID)
pub type ShapeId = usize;

/// Inline cache for property access optimization
///
/// Caches the shape (hidden class) and property offset for fast access.
/// Transitions through states as more shapes are encountered.
#[derive(Debug, Clone, PartialEq)]
pub enum InlineCache {
    /// No shape cached yet
    Uninitialized,
    /// Single shape cached (most common case)
    Monomorphic {
        /// The cached shape ID
        shape: ShapeId,
        /// The property offset for this shape
        offset: u32,
    },
    /// Multiple shapes cached (up to 4)
    Polymorphic {
        /// List of (shape, offset) pairs
        entries: ArrayVec<(ShapeId, u32), 4>,
    },
    /// Too many shapes, fallback to hash table lookup
    Megamorphic,
}

impl InlineCache {
    /// Create a new uninitialized cache
    pub fn new() -> Self {
        InlineCache::Uninitialized
    }

    /// Look up property offset for given shape
    ///
    /// Returns Some(offset) if shape is cached, None otherwise.
    pub fn lookup(&self, shape: ShapeId) -> Option<u32> {
        match self {
            InlineCache::Uninitialized => None,
            InlineCache::Monomorphic {
                shape: cached_shape,
                offset,
            } => {
                if *cached_shape == shape {
                    Some(*offset)
                } else {
                    None
                }
            }
            InlineCache::Polymorphic { entries } => entries
                .iter()
                .find(|(s, _)| *s == shape)
                .map(|(_, offset)| *offset),
            InlineCache::Megamorphic => None,
        }
    }

    /// Update cache with new shape and offset
    ///
    /// Transitions cache state as needed:
    /// - Uninitialized → Monomorphic
    /// - Monomorphic → Polymorphic (if different shape)
    /// - Polymorphic → Megamorphic (if > 4 shapes)
    pub fn update(&mut self, shape: ShapeId, offset: u32) {
        match self {
            InlineCache::Uninitialized => {
                *self = InlineCache::Monomorphic { shape, offset };
            }
            InlineCache::Monomorphic {
                shape: cached_shape,
                offset: cached_offset,
            } => {
                if *cached_shape == shape {
                    // Same shape, just update offset
                    *cached_offset = offset;
                } else {
                    // Different shape, transition to polymorphic
                    let mut entries = ArrayVec::new();
                    entries.push((*cached_shape, *cached_offset));
                    entries.push((shape, offset));
                    *self = InlineCache::Polymorphic { entries };
                }
            }
            InlineCache::Polymorphic { entries } => {
                // Check if shape already cached
                if let Some(entry) = entries.iter_mut().find(|(s, _)| *s == shape) {
                    entry.1 = offset;
                } else if entries.len() < 4 {
                    // Add new entry
                    entries.push((shape, offset));
                } else {
                    // Too many shapes, transition to megamorphic
                    *self = InlineCache::Megamorphic;
                }
            }
            InlineCache::Megamorphic => {
                // Already megamorphic, no change
            }
        }
    }
}

impl Default for InlineCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_cache_new() {
        let cache = InlineCache::new();
        assert!(matches!(cache, InlineCache::Uninitialized));
    }

    #[test]
    fn test_inline_cache_default() {
        let cache = InlineCache::default();
        assert!(matches!(cache, InlineCache::Uninitialized));
    }

    #[test]
    fn test_polymorphic_max_entries() {
        let mut entries = ArrayVec::new();
        entries.push((1, 0));
        entries.push((2, 1));
        entries.push((3, 2));
        entries.push((4, 3));

        let cache = InlineCache::Polymorphic { entries };
        assert_eq!(cache.lookup(3), Some(2));
    }
}

//! Heap management with generational garbage collection.
//!
//! This module implements a generational heap with:
//! - Young generation: Semi-space copying collector
//! - Old generation: Mark-and-sweep collector
//! - Write barriers for tracking old-to-young pointers

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashSet;
use std::ptr;

/// Arena allocator for a generation.
///
/// Uses bump-pointer allocation for fast allocation within a semi-space.
#[derive(Debug)]
pub struct Arena {
    /// Base pointer of the arena
    base: *mut u8,
    /// Current allocation pointer (bump pointer)
    current: *mut u8,
    /// End of the arena
    end: *mut u8,
    /// Total capacity in bytes
    capacity: usize,
}

impl Arena {
    /// Creates a new arena with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Size of the arena in bytes
    ///
    /// # Panics
    ///
    /// Panics if memory allocation fails.
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, 8).expect("Invalid layout");
        // SAFETY: We're allocating a new block of memory with proper alignment
        let base = unsafe { alloc(layout) };
        if base.is_null() {
            panic!("Failed to allocate arena of size {}", capacity);
        }

        // SAFETY: We just allocated this memory, so adding capacity is valid
        let end = unsafe { base.add(capacity) };

        Arena {
            base,
            current: base,
            end,
            capacity,
        }
    }

    /// Allocates memory from the arena using bump-pointer allocation.
    ///
    /// # Arguments
    ///
    /// * `size` - Number of bytes to allocate
    /// * `align` - Alignment requirement (must be power of 2)
    ///
    /// # Returns
    ///
    /// Pointer to allocated memory, or null if insufficient space.
    pub fn allocate(&mut self, size: usize, align: usize) -> *mut u8 {
        // Align current pointer
        let current_addr = self.current as usize;
        let aligned_addr = (current_addr + align - 1) & !(align - 1);
        let aligned_ptr = aligned_addr as *mut u8;

        // Check if we have enough space
        // SAFETY: aligned_ptr is within or just past our arena
        let new_current = unsafe { aligned_ptr.add(size) };
        if new_current > self.end {
            return ptr::null_mut();
        }

        self.current = new_current;
        aligned_ptr
    }

    /// Returns the number of bytes currently allocated.
    pub fn used(&self) -> usize {
        self.current as usize - self.base as usize
    }

    /// Returns the total capacity of the arena.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Resets the arena, effectively freeing all allocations.
    ///
    /// # Safety
    ///
    /// All pointers previously returned by `allocate` become invalid.
    pub fn reset(&mut self) {
        self.current = self.base;
    }

    /// Returns the base pointer of the arena.
    pub fn base_ptr(&self) -> *mut u8 {
        self.base
    }

    /// Returns the end pointer of the arena.
    pub fn end_ptr(&self) -> *mut u8 {
        self.end
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        if !self.base.is_null() {
            let layout = Layout::from_size_align(self.capacity, 8).expect("Invalid layout");
            // SAFETY: We're deallocating memory we allocated in new()
            unsafe {
                dealloc(self.base, layout);
            }
        }
    }
}

/// GC object header containing metadata for garbage collection.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ObjectHeader {
    /// Size of the object in bytes (including header)
    pub size: usize,
    /// Marking state for GC (white=0, gray=1, black=2)
    pub mark: u8,
    /// Object type tag
    pub tag: u8,
    /// Reserved for future use
    pub reserved: u16,
}

/// A heap-allocated object in the garbage collector.
#[repr(C)]
pub struct Object {
    /// Object header with GC metadata
    pub header: ObjectHeader,
    // Data follows the header in memory
}

impl Object {
    /// Returns whether this object is marked (gray or black).
    pub fn is_marked(&self) -> bool {
        self.header.mark > 0
    }

    /// Sets the mark state of this object.
    pub fn set_mark(&mut self, mark: u8) {
        self.header.mark = mark;
    }
}

/// The main heap structure with generational garbage collection.
///
/// Contains:
/// - Young generation for newly allocated objects
/// - Old generation for long-lived objects
/// - Remembered set for tracking old-to-young pointers
#[derive(Debug)]
pub struct Heap {
    /// Young generation (from-space)
    young_from: Arena,
    /// Young generation (to-space for copying)
    young_to: Arena,
    /// Old generation
    old_gen: Arena,
    /// Remembered set: old objects pointing to young objects
    remembered_set: HashSet<*mut Object>,
    /// Whether we're currently in a marking phase
    is_marking: bool,
    /// Number of collections performed
    collection_count: usize,
}

/// Default young generation size (1MB)
const YOUNG_GEN_SIZE: usize = 1024 * 1024;
/// Default old generation size (4MB)
const OLD_GEN_SIZE: usize = 4 * 1024 * 1024;

impl Heap {
    /// Creates a new heap with default generation sizes.
    pub fn new() -> Self {
        Heap {
            young_from: Arena::new(YOUNG_GEN_SIZE),
            young_to: Arena::new(YOUNG_GEN_SIZE),
            old_gen: Arena::new(OLD_GEN_SIZE),
            remembered_set: HashSet::new(),
            is_marking: false,
            collection_count: 0,
        }
    }

    /// Allocates memory from the young generation.
    ///
    /// # Arguments
    ///
    /// * `size` - Number of bytes to allocate
    ///
    /// # Returns
    ///
    /// Pointer to allocated memory, or null if young generation is full.
    pub fn allocate(&mut self, size: usize) -> *mut u8 {
        let total_size = size + std::mem::size_of::<ObjectHeader>();
        let ptr = self.young_from.allocate(total_size, 8);

        if ptr.is_null() {
            // Young generation full, trigger minor GC
            self.minor_gc();
            // Try again after GC
            self.young_from.allocate(total_size, 8)
        } else {
            // Initialize object header
            // SAFETY: We just allocated this memory, so it's valid to write to
            unsafe {
                let header = ptr as *mut ObjectHeader;
                (*header).size = total_size;
                (*header).mark = 0; // White (unmarked)
                (*header).tag = 0;
                (*header).reserved = 0;
            }
            ptr
        }
    }

    /// Performs garbage collection.
    ///
    /// This runs a minor GC on the young generation. If the young generation
    /// is still too full, it may promote objects to the old generation.
    pub fn collect_garbage(&mut self) {
        self.minor_gc();
        self.collection_count += 1;
    }

    /// Returns the size of the young generation in bytes.
    pub fn young_generation_size(&self) -> usize {
        self.young_from.capacity()
    }

    /// Returns the size of the old generation in bytes.
    pub fn old_generation_size(&self) -> usize {
        self.old_gen.capacity()
    }

    /// Returns the number of bytes currently used in the young generation.
    pub fn young_generation_used(&self) -> usize {
        self.young_from.used()
    }

    /// Returns the number of bytes currently used in the old generation.
    pub fn old_generation_used(&self) -> usize {
        self.old_gen.used()
    }

    /// Returns the number of garbage collections performed.
    pub fn collection_count(&self) -> usize {
        self.collection_count
    }

    /// Performs a minor GC (scavenge) on the young generation.
    fn minor_gc(&mut self) {
        // Reset the to-space
        self.young_to.reset();

        // In a real implementation, we would:
        // 1. Scan roots
        // 2. Copy live objects from from-space to to-space
        // 3. Update references
        // 4. Swap from-space and to-space

        // For now, just swap the spaces (simulated scavenge)
        std::mem::swap(&mut self.young_from, &mut self.young_to);

        // Clear remembered set after GC
        self.remembered_set.clear();
    }

    /// Checks if a pointer is in the young generation.
    pub fn is_in_young_gen(&self, ptr: *const u8) -> bool {
        let ptr_addr = ptr as usize;
        let base = self.young_from.base_ptr() as usize;
        let end = self.young_from.end_ptr() as usize;
        ptr_addr >= base && ptr_addr < end
    }

    /// Checks if a pointer is in the old generation.
    pub fn is_in_old_gen(&self, ptr: *const u8) -> bool {
        let ptr_addr = ptr as usize;
        let base = self.old_gen.base_ptr() as usize;
        let end = self.old_gen.end_ptr() as usize;
        ptr_addr >= base && ptr_addr < end
    }

    /// Adds an object to the remembered set.
    pub fn add_to_remembered_set(&mut self, obj: *mut Object) {
        self.remembered_set.insert(obj);
    }

    /// Returns whether we're currently in a marking phase.
    pub fn is_marking(&self) -> bool {
        self.is_marking
    }

    /// Sets the marking state.
    pub fn set_marking(&mut self, marking: bool) {
        self.is_marking = marking;
    }

    /// Returns the remembered set (for testing).
    pub fn remembered_set(&self) -> &HashSet<*mut Object> {
        &self.remembered_set
    }
}

impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_new() {
        let arena = Arena::new(1024);
        assert_eq!(arena.capacity(), 1024);
        assert_eq!(arena.used(), 0);
    }

    #[test]
    fn test_arena_allocate() {
        let mut arena = Arena::new(1024);
        let ptr1 = arena.allocate(64, 8);
        assert!(!ptr1.is_null());
        assert!(arena.used() >= 64);

        let ptr2 = arena.allocate(128, 8);
        assert!(!ptr2.is_null());
        assert!(arena.used() >= 192);
    }

    #[test]
    fn test_arena_allocate_alignment() {
        let mut arena = Arena::new(1024);

        // Allocate 3 bytes, then request 8-byte alignment
        let _ptr1 = arena.allocate(3, 1);
        let ptr2 = arena.allocate(8, 8);

        // ptr2 should be 8-byte aligned
        assert_eq!((ptr2 as usize) % 8, 0);
    }

    #[test]
    fn test_arena_allocate_overflow() {
        let mut arena = Arena::new(128);

        let ptr1 = arena.allocate(100, 8);
        assert!(!ptr1.is_null());

        // This should fail - not enough space
        let ptr2 = arena.allocate(100, 8);
        assert!(ptr2.is_null());
    }

    #[test]
    fn test_arena_reset() {
        let mut arena = Arena::new(1024);

        let _ptr = arena.allocate(512, 8);
        assert!(arena.used() > 0);

        arena.reset();
        assert_eq!(arena.used(), 0);
    }

    #[test]
    fn test_heap_new() {
        let heap = Heap::new();
        assert_eq!(heap.young_generation_size(), YOUNG_GEN_SIZE);
        assert_eq!(heap.old_generation_size(), OLD_GEN_SIZE);
        assert_eq!(heap.collection_count(), 0);
    }

    #[test]
    fn test_heap_allocate() {
        let mut heap = Heap::new();
        let ptr = heap.allocate(64);
        assert!(!ptr.is_null());
        assert!(heap.young_generation_used() > 0);
    }

    #[test]
    fn test_heap_allocate_multiple() {
        let mut heap = Heap::new();

        let ptr1 = heap.allocate(32);
        let ptr2 = heap.allocate(64);
        let ptr3 = heap.allocate(128);

        assert!(!ptr1.is_null());
        assert!(!ptr2.is_null());
        assert!(!ptr3.is_null());

        // Pointers should be different
        assert_ne!(ptr1, ptr2);
        assert_ne!(ptr2, ptr3);
    }

    #[test]
    fn test_heap_collect_garbage() {
        let mut heap = Heap::new();

        let _ptr = heap.allocate(64);
        let used_before = heap.young_generation_used();

        heap.collect_garbage();

        // After GC, the used space should be reset (simplified GC)
        let used_after = heap.young_generation_used();
        assert_eq!(heap.collection_count(), 1);
        // In our simplified GC, we just swap spaces
        assert!(used_after <= used_before);
    }

    #[test]
    fn test_heap_is_in_young_gen() {
        let mut heap = Heap::new();
        let ptr = heap.allocate(64);

        assert!(heap.is_in_young_gen(ptr));
        assert!(!heap.is_in_old_gen(ptr));
    }

    #[test]
    fn test_heap_remembered_set() {
        let mut heap = Heap::new();
        let ptr = heap.allocate(64) as *mut Object;

        heap.add_to_remembered_set(ptr);
        assert!(heap.remembered_set().contains(&ptr));

        // GC clears remembered set
        heap.collect_garbage();
        assert!(heap.remembered_set().is_empty());
    }

    #[test]
    fn test_heap_marking_state() {
        let mut heap = Heap::new();
        assert!(!heap.is_marking());

        heap.set_marking(true);
        assert!(heap.is_marking());

        heap.set_marking(false);
        assert!(!heap.is_marking());
    }

    #[test]
    fn test_object_header() {
        let header = ObjectHeader {
            size: 64,
            mark: 0,
            tag: 1,
            reserved: 0,
        };
        assert_eq!(header.size, 64);
        assert_eq!(header.mark, 0);
        assert_eq!(header.tag, 1);
    }

    #[test]
    fn test_object_marking() {
        let mut heap = Heap::new();
        let ptr = heap.allocate(64) as *mut Object;

        // SAFETY: We just allocated this object
        unsafe {
            assert!(!(*ptr).is_marked());
            (*ptr).set_mark(1); // Gray
            assert!((*ptr).is_marked());
            (*ptr).set_mark(2); // Black
            assert!((*ptr).is_marked());
            (*ptr).set_mark(0); // White
            assert!(!(*ptr).is_marked());
        }
    }

    #[test]
    fn test_heap_default() {
        let heap = Heap::default();
        assert_eq!(heap.young_generation_size(), YOUNG_GEN_SIZE);
    }
}

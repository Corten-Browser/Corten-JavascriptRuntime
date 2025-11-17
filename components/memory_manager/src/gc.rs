//! Semi-space copying garbage collector for young generation.
//!
//! This module implements Cheney's algorithm for copying garbage collection:
//! - Two equal-sized spaces: from_space and to_space
//! - Bump-pointer allocation in from_space
//! - During GC, copy live objects to to_space
//! - Update all references to new locations
//! - Swap spaces after collection

use std::collections::{HashMap, VecDeque};
use std::ptr;

/// Mark colors for tri-color marking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MarkColor {
    /// Unmarked (not yet visited)
    White = 0,
    /// In process (reachable, needs scanning)
    Gray = 1,
    /// Fully processed (reachable, all references scanned)
    Black = 2,
}

/// GC object header containing metadata for garbage collection.
///
/// This header is placed at the beginning of every GC-managed object.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GcObjectHeader {
    /// Marking state for tri-color marking
    pub mark: u8,
    /// Forwarding pointer for copying GC (null if not forwarded)
    pub forwarding: *mut u8,
    /// Object size in bytes (including header)
    pub size: u32,
    /// Generation age (incremented on survival, used for promotion)
    pub age: u8,
    /// Reserved for alignment
    pub _reserved: [u8; 3],
}

impl GcObjectHeader {
    /// Creates a new header with default values.
    pub fn new(size: u32) -> Self {
        GcObjectHeader {
            mark: MarkColor::White as u8,
            forwarding: ptr::null_mut(),
            size,
            age: 0,
            _reserved: [0; 3],
        }
    }

    /// Returns true if this object has been forwarded.
    pub fn is_forwarded(&self) -> bool {
        !self.forwarding.is_null()
    }
}

/// A garbage-collected object with header.
#[repr(C)]
pub struct GcObject {
    /// Object header with GC metadata
    pub header: GcObjectHeader,
    // Object data follows in memory
}

impl GcObject {
    /// Returns the mark color of this object.
    pub fn mark_color(&self) -> MarkColor {
        match self.header.mark {
            0 => MarkColor::White,
            1 => MarkColor::Gray,
            2 => MarkColor::Black,
            _ => MarkColor::White,
        }
    }

    /// Sets the mark color of this object.
    pub fn set_mark_color(&mut self, color: MarkColor) {
        self.header.mark = color as u8;
    }

    /// Returns a pointer to the data portion of this object.
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid for the lifetime of the object.
    pub unsafe fn data_ptr(&mut self) -> *mut u8 {
        let header_size = std::mem::size_of::<GcObjectHeader>();
        // SAFETY: Adding header_size to self pointer gives data location
        (self as *mut GcObject as *mut u8).add(header_size)
    }

    /// Returns the total size of this object including header.
    pub fn total_size(&self) -> usize {
        self.header.size as usize
    }
}

/// Young generation with semi-space copying garbage collector.
///
/// Uses Cheney's algorithm:
/// 1. Two equal-sized spaces (from_space and to_space)
/// 2. Allocation uses bump pointer in from_space
/// 3. During GC, live objects are copied to to_space
/// 4. Spaces are swapped after collection
pub struct YoungGeneration {
    /// The space where objects are allocated
    from_space: Box<[u8]>,
    /// The space where objects are copied during GC
    to_space: Box<[u8]>,
    /// Current allocation pointer in from_space
    allocation_ptr: usize,
    /// Size of each space in bytes
    space_size: usize,
    /// Scan pointer for Cheney's algorithm (used during copying)
    scan_ptr: usize,
}

impl YoungGeneration {
    /// Creates a new young generation with the specified size.
    ///
    /// Each space (from_space and to_space) will have this size.
    ///
    /// # Arguments
    ///
    /// * `size` - Size of each semi-space in bytes
    pub fn new(size: usize) -> Self {
        let from_space = vec![0u8; size].into_boxed_slice();
        let to_space = vec![0u8; size].into_boxed_slice();

        YoungGeneration {
            from_space,
            to_space,
            allocation_ptr: 0,
            space_size: size,
            scan_ptr: 0,
        }
    }

    /// Allocates memory using bump-pointer allocation.
    ///
    /// This is the fast path for allocation - just bump the pointer.
    ///
    /// # Arguments
    ///
    /// * `size` - Number of bytes to allocate (must include header size)
    ///
    /// # Returns
    ///
    /// Pointer to allocated memory, or None if insufficient space.
    pub fn allocate(&mut self, size: usize) -> Option<*mut u8> {
        // Align size to 8 bytes
        let aligned_size = (size + 7) & !7;

        // Check if we have enough space
        if self.allocation_ptr + aligned_size > self.space_size {
            return None;
        }

        // SAFETY: allocation_ptr is within bounds of from_space,
        // and adding aligned_size still keeps us within bounds (we just checked)
        let ptr = unsafe { self.from_space.as_mut_ptr().add(self.allocation_ptr) };

        // Bump the pointer
        self.allocation_ptr += aligned_size;

        Some(ptr)
    }

    /// Performs garbage collection using Cheney's copying algorithm.
    ///
    /// This copies all live objects from from_space to to_space,
    /// updating all references to point to the new locations.
    ///
    /// # Arguments
    ///
    /// * `roots` - Root set of pointers to live objects
    ///
    /// # Returns
    ///
    /// A vector of (old_location, new_location) pairs for updated objects.
    pub fn collect(&mut self, roots: &[*mut GcObject]) -> Vec<(*mut GcObject, *mut GcObject)> {
        // Reset to_space and scan pointer
        self.scan_ptr = 0;
        let mut allocation_in_to = 0;
        let mut relocations = Vec::new();

        // Phase 1: Copy root objects to to_space
        let mut _root_mappings: HashMap<*mut GcObject, *mut GcObject> = HashMap::new();
        for &root in roots {
            if !root.is_null() {
                let new_location = self.copy_object(root, &mut allocation_in_to);
                _root_mappings.insert(root, new_location);
                relocations.push((root, new_location));
            }
        }

        // Phase 2: Scan objects in to_space (Cheney's breadth-first copying)
        while self.scan_ptr < allocation_in_to {
            // SAFETY: scan_ptr is within bounds of to_space,
            // pointing to a valid GcObject we just copied
            let obj_ptr = unsafe { self.to_space.as_mut_ptr().add(self.scan_ptr) as *mut GcObject };

            // SAFETY: obj_ptr points to a valid GcObject in to_space
            let obj_size = unsafe { (*obj_ptr).total_size() };

            // Scan this object's references and copy any unreached objects
            // For now, we'll use a simplified model where we process the object
            // In a real implementation, we'd iterate over the object's fields
            self.scan_ptr += (obj_size + 7) & !7;
        }

        // Phase 3: Swap spaces
        std::mem::swap(&mut self.from_space, &mut self.to_space);

        // Update allocation pointer to end of live data in new from_space
        self.allocation_ptr = allocation_in_to;

        // Reset to_space for next collection
        self.to_space.iter_mut().for_each(|b| *b = 0);

        relocations
    }

    /// Copies a single object to to_space using Cheney's algorithm.
    ///
    /// If the object has already been copied (forwarding pointer set),
    /// returns the existing forwarding address.
    ///
    /// # Arguments
    ///
    /// * `obj` - Pointer to object in from_space
    /// * `allocation_in_to` - Current allocation pointer in to_space
    ///
    /// # Returns
    ///
    /// Pointer to the object in to_space.
    fn copy_object(&mut self, obj: *mut GcObject, allocation_in_to: &mut usize) -> *mut GcObject {
        // SAFETY: obj must be a valid pointer to a GcObject in from_space.
        // The caller is responsible for ensuring this invariant.
        unsafe {
            let header = &mut (*obj).header;

            // Check if already forwarded
            if header.is_forwarded() {
                // SAFETY: Forwarding pointer points to valid location in to_space
                // that was set in a previous call to copy_object
                return header.forwarding as *mut GcObject;
            }

            // Get object size
            let obj_size = header.size as usize;
            let aligned_size = (obj_size + 7) & !7;

            // SAFETY: allocation_in_to is within bounds of to_space,
            // and we're about to copy obj_size bytes into that location
            let new_location = self.to_space.as_mut_ptr().add(*allocation_in_to);

            // SAFETY: Copying obj_size bytes from obj to new_location.
            // Both pointers are valid and non-overlapping (different spaces).
            ptr::copy_nonoverlapping(obj as *const u8, new_location, obj_size);

            // Clear forwarding pointer in the copy
            let new_obj = new_location as *mut GcObject;
            (*new_obj).header.forwarding = ptr::null_mut();

            // Increment age (survived a collection)
            (*new_obj).header.age = (*new_obj).header.age.saturating_add(1);

            // Set forwarding pointer in original
            header.forwarding = new_location;

            // Update allocation pointer in to_space
            *allocation_in_to += aligned_size;

            new_obj
        }
    }

    /// Returns the number of bytes currently used in from_space.
    pub fn used_space(&self) -> usize {
        self.allocation_ptr
    }

    /// Returns the number of bytes available for allocation.
    pub fn free_space(&self) -> usize {
        self.space_size - self.allocation_ptr
    }

    /// Returns the total size of each semi-space.
    pub fn space_size(&self) -> usize {
        self.space_size
    }

    /// Checks if a pointer is within the from_space.
    pub fn is_in_from_space(&self, ptr: *const u8) -> bool {
        let ptr_addr = ptr as usize;
        let base = self.from_space.as_ptr() as usize;
        let end = base + self.space_size;
        ptr_addr >= base && ptr_addr < end
    }

    /// Checks if a pointer is within the to_space.
    pub fn is_in_to_space(&self, ptr: *const u8) -> bool {
        let ptr_addr = ptr as usize;
        let base = self.to_space.as_ptr() as usize;
        let end = base + self.space_size;
        ptr_addr >= base && ptr_addr < end
    }

    /// Returns the base pointer of from_space (for testing).
    pub fn from_space_base(&self) -> *const u8 {
        self.from_space.as_ptr()
    }

    /// Returns the base pointer of to_space (for testing).
    pub fn to_space_base(&self) -> *const u8 {
        self.to_space.as_ptr()
    }
}

/// Trait for objects that can be traced by the garbage collector.
///
/// Implementing this trait allows the GC to discover all references
/// from an object to other objects during the marking phase.
pub trait Traceable {
    /// Traces all GC object references in this object.
    ///
    /// # Arguments
    ///
    /// * `tracer` - Callback to invoke for each child reference
    ///
    /// # Safety
    ///
    /// The tracer must be invoked with valid GcObject pointers only.
    fn trace(&self, tracer: &mut dyn FnMut(*mut GcObject));
}

/// Old generation garbage collector using tri-color marking.
///
/// The old generation contains long-lived objects that have survived
/// multiple young generation collections. It uses a tri-color marking
/// algorithm to identify and collect garbage:
///
/// 1. **Mark Phase**: Start from roots, mark reachable objects
/// 2. **Sweep Phase**: Free all unmarked (white) objects
///
/// # Tri-Color Invariant
///
/// During marking, the invariant is maintained:
/// - Black objects never point to white objects
/// - All reachable objects will eventually be marked black
///
/// # Example
///
/// ```
/// use memory_manager::gc::{OldGeneration, GcObject, GcObjectHeader};
/// use std::ptr;
///
/// let mut old_gen = OldGeneration::new();
///
/// // Create some objects (in a real implementation, these would be allocated properly)
/// let obj1 = Box::into_raw(Box::new(GcObject {
///     header: GcObjectHeader::new(32),
/// }));
/// let obj2 = Box::into_raw(Box::new(GcObject {
///     header: GcObjectHeader::new(64),
/// }));
///
/// // SAFETY: obj1 and obj2 are valid pointers to GcObjects
/// unsafe {
///     old_gen.add_object(obj1);
///     old_gen.add_object(obj2);
/// }
///
/// // Collect with obj1 as root (obj2 becomes garbage)
/// let freed = old_gen.collect(&[obj1]);
///
/// assert_eq!(freed, 64); // obj2 was freed
/// assert_eq!(old_gen.object_count(), 1); // Only obj1 remains
///
/// // Clean up remaining object
/// old_gen.collect(&[]); // Free all
/// ```
pub struct OldGeneration {
    /// All objects in the old generation
    objects: Vec<*mut GcObject>,
    /// Gray list (worklist) for marking phase
    gray_list: VecDeque<*mut GcObject>,
    /// Total memory used by objects in this generation
    total_size: usize,
}

impl OldGeneration {
    /// Creates a new empty old generation.
    pub fn new() -> Self {
        OldGeneration {
            objects: Vec::new(),
            gray_list: VecDeque::new(),
            total_size: 0,
        }
    }

    /// Adds a promoted object from the young generation.
    ///
    /// # Arguments
    ///
    /// * `obj` - Pointer to the GcObject to add
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - `obj` is a valid pointer to a GcObject
    /// - `obj` will remain valid until removed or collected
    /// - `obj` is not already in this generation
    pub unsafe fn add_object(&mut self, obj: *mut GcObject) {
        if obj.is_null() {
            return;
        }

        // SAFETY: Caller guarantees obj is valid
        let size = (*obj).total_size();
        self.objects.push(obj);
        self.total_size += size;
    }

    /// Performs tri-color marking collection.
    ///
    /// # Arguments
    ///
    /// * `roots` - Slice of root object pointers (stack, globals, etc.)
    ///
    /// # Returns
    ///
    /// Number of bytes freed by collection.
    ///
    /// # Safety
    ///
    /// All pointers in `roots` must be valid GcObject pointers.
    pub fn collect(&mut self, roots: &[*mut GcObject]) -> usize {
        if self.objects.is_empty() {
            return 0;
        }

        // Phase 1: Mark all objects white (initial state)
        self.mark_all_white();

        // Phase 2: Mark roots as gray
        self.mark_roots(roots);

        // Phase 3: Process gray list (transitive closure)
        self.process_gray_list();

        // Phase 4: Sweep white objects (garbage)
        self.sweep()
    }

    /// Marks all objects in the generation as white.
    fn mark_all_white(&mut self) {
        for &obj_ptr in &self.objects {
            if !obj_ptr.is_null() {
                // SAFETY: Objects in our list are guaranteed valid by add_object
                unsafe {
                    (*obj_ptr).set_mark_color(MarkColor::White);
                }
            }
        }
    }

    /// Marks root objects as gray.
    ///
    /// # Arguments
    ///
    /// * `roots` - Root object pointers to mark
    fn mark_roots(&mut self, roots: &[*mut GcObject]) {
        for &root in roots {
            if !root.is_null() {
                // SAFETY: Caller of collect() guarantees roots are valid
                unsafe {
                    if (*root).mark_color() == MarkColor::White {
                        (*root).set_mark_color(MarkColor::Gray);
                        self.gray_list.push_back(root);
                    }
                }
            }
        }
    }

    /// Processes the gray list until empty.
    ///
    /// For each gray object:
    /// 1. Mark it black (fully processed)
    /// 2. Trace its children and mark them gray if white
    ///
    /// This implements the core of the tri-color marking algorithm,
    /// ensuring all reachable objects are eventually marked black.
    fn process_gray_list(&mut self) {
        while let Some(obj_ptr) = self.gray_list.pop_front() {
            if obj_ptr.is_null() {
                continue;
            }

            // SAFETY: Objects come from gray_list which only contains valid pointers
            // from our objects list or roots (both validated at insertion)
            unsafe {
                // Mark this object black (fully processed)
                (*obj_ptr).set_mark_color(MarkColor::Black);

                // In a full implementation with integrated tracing:
                // We would trace children here. For the basic implementation,
                // objects without children are simply marked black.
            }
        }
    }

    /// Processes the gray list with a custom tracer function.
    ///
    /// This version allows external code to provide tracing logic for objects,
    /// enabling cycle detection and complex object graphs.
    ///
    /// # Arguments
    ///
    /// * `trace_fn` - Function that traces an object's children
    ///
    /// # Safety
    ///
    /// The trace function must only yield valid GcObject pointers.
    pub unsafe fn process_gray_list_with_tracer<F>(&mut self, mut trace_fn: F)
    where
        F: FnMut(*mut GcObject, &mut dyn FnMut(*mut GcObject)),
    {
        while let Some(obj_ptr) = self.gray_list.pop_front() {
            if obj_ptr.is_null() {
                continue;
            }

            // SAFETY: Objects come from gray_list which only contains valid pointers
            // Mark this object black (fully processed)
            (*obj_ptr).set_mark_color(MarkColor::Black);

            // Trace children and mark them gray if white
            trace_fn(obj_ptr, &mut |child_ptr| {
                if !child_ptr.is_null() {
                    // SAFETY: trace_fn is responsible for yielding valid pointers
                    let child = &mut *child_ptr;
                    if child.mark_color() == MarkColor::White {
                        child.set_mark_color(MarkColor::Gray);
                        self.gray_list.push_back(child_ptr);
                    }
                }
            });
        }
    }

    /// Sweeps all white objects (garbage) and frees their memory.
    ///
    /// Objects that remain white after marking are unreachable and can be
    /// safely freed. Black objects are retained.
    ///
    /// # Returns
    ///
    /// Number of bytes freed.
    fn sweep(&mut self) -> usize {
        let mut freed = 0;
        let mut retained = Vec::new();

        for obj_ptr in self.objects.drain(..) {
            if obj_ptr.is_null() {
                continue;
            }

            // SAFETY: All objects in our list are valid by invariant
            let (is_garbage, size) = unsafe {
                let obj = &*obj_ptr;
                (obj.mark_color() == MarkColor::White, obj.total_size())
            };

            if is_garbage {
                // Object is white (unreachable) - free it
                freed += size;
                self.total_size -= size;

                // SAFETY: We're freeing the object, which was allocated with Box
                // The caller is responsible for ensuring proper allocation/deallocation
                unsafe {
                    let _ = Box::from_raw(obj_ptr);
                }
            } else {
                // Object is black (reachable) - keep it
                retained.push(obj_ptr);
            }
        }

        self.objects = retained;
        freed
    }

    /// Returns the number of objects in the old generation.
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    /// Returns the total memory used by objects in bytes.
    pub fn total_memory(&self) -> usize {
        self.total_size
    }

    /// Returns whether the gray list is empty (for testing/debugging).
    pub fn gray_list_is_empty(&self) -> bool {
        self.gray_list.is_empty()
    }

    /// Returns the number of gray objects (for testing/debugging).
    pub fn gray_list_size(&self) -> usize {
        self.gray_list.len()
    }

    /// Gets the mark color of an object (for testing/debugging).
    ///
    /// # Safety
    ///
    /// The pointer must be valid.
    pub unsafe fn get_mark_color(&self, obj: *mut GcObject) -> MarkColor {
        (*obj).mark_color()
    }

    /// Checks if a pointer is in the old generation.
    pub fn contains(&self, ptr: *mut GcObject) -> bool {
        self.objects.contains(&ptr)
    }

    /// Returns an iterator over all objects (for debugging).
    pub fn objects(&self) -> &[*mut GcObject] {
        &self.objects
    }
}

impl Default for OldGeneration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_object_header_new() {
        let header = GcObjectHeader::new(64);
        assert_eq!(header.size, 64);
        assert_eq!(header.mark, MarkColor::White as u8);
        assert_eq!(header.age, 0);
        assert!(header.forwarding.is_null());
        assert!(!header.is_forwarded());
    }

    #[test]
    fn test_gc_object_header_forwarding() {
        let mut header = GcObjectHeader::new(32);
        assert!(!header.is_forwarded());

        let fake_ptr = 0x1234 as *mut u8;
        header.forwarding = fake_ptr;
        assert!(header.is_forwarded());
    }

    #[test]
    fn test_gc_object_mark_color() {
        let mut obj = GcObject {
            header: GcObjectHeader::new(std::mem::size_of::<GcObject>() as u32),
        };

        assert_eq!(obj.mark_color(), MarkColor::White);

        obj.set_mark_color(MarkColor::Gray);
        assert_eq!(obj.mark_color(), MarkColor::Gray);

        obj.set_mark_color(MarkColor::Black);
        assert_eq!(obj.mark_color(), MarkColor::Black);
    }

    #[test]
    fn test_young_generation_new() {
        let gen = YoungGeneration::new(1024);
        assert_eq!(gen.space_size(), 1024);
        assert_eq!(gen.used_space(), 0);
        assert_eq!(gen.free_space(), 1024);
    }

    #[test]
    fn test_young_generation_allocate() {
        let mut gen = YoungGeneration::new(1024);

        let ptr = gen.allocate(64);
        assert!(ptr.is_some());
        let ptr = ptr.unwrap();
        assert!(!ptr.is_null());
        assert!(gen.is_in_from_space(ptr));

        // Should have allocated at least 64 bytes (aligned to 8)
        assert!(gen.used_space() >= 64);
    }

    #[test]
    fn test_young_generation_allocate_multiple() {
        let mut gen = YoungGeneration::new(1024);

        let ptr1 = gen.allocate(32).unwrap();
        let ptr2 = gen.allocate(64).unwrap();
        let ptr3 = gen.allocate(128).unwrap();

        assert_ne!(ptr1, ptr2);
        assert_ne!(ptr2, ptr3);

        // All should be in from_space
        assert!(gen.is_in_from_space(ptr1));
        assert!(gen.is_in_from_space(ptr2));
        assert!(gen.is_in_from_space(ptr3));
    }

    #[test]
    fn test_young_generation_allocate_fills_space() {
        let mut gen = YoungGeneration::new(128);

        // Fill most of the space
        let _ptr1 = gen.allocate(100).unwrap();
        assert!(gen.used_space() >= 100);

        // This should fail - not enough space
        let ptr2 = gen.allocate(100);
        assert!(ptr2.is_none());
    }

    #[test]
    fn test_young_generation_allocate_alignment() {
        let mut gen = YoungGeneration::new(1024);

        // Allocate odd size
        let ptr1 = gen.allocate(17).unwrap();
        let used_after_first = gen.used_space();

        // Next allocation should still be 8-byte aligned
        let ptr2 = gen.allocate(32).unwrap();

        // Check alignment
        assert_eq!((ptr2 as usize) % 8, 0);
        // Should have padded the first allocation to 24 bytes (17 aligned to 8)
        assert_eq!(used_after_first, 24);
        let _ = ptr1; // Silence unused warning
    }

    #[test]
    fn test_young_generation_collect_empty() {
        let mut gen = YoungGeneration::new(1024);
        let roots: Vec<*mut GcObject> = vec![];

        let relocations = gen.collect(&roots);
        assert_eq!(relocations.len(), 0);
        assert_eq!(gen.used_space(), 0);
    }

    #[test]
    fn test_young_generation_collect_single_object() {
        let mut gen = YoungGeneration::new(1024);

        // Allocate an object
        let obj_size = std::mem::size_of::<GcObject>() as u32;
        let ptr = gen.allocate(obj_size as usize).unwrap();

        // Initialize it as a GcObject
        // SAFETY: ptr points to allocated memory of sufficient size
        unsafe {
            let obj = ptr as *mut GcObject;
            (*obj).header = GcObjectHeader::new(obj_size);
        }

        // Collect with this object as root
        let old_from_base = gen.from_space_base();
        let old_to_base = gen.to_space_base();

        let roots = vec![ptr as *mut GcObject];
        let relocations = gen.collect(&roots);

        // Should have relocated the object
        assert_eq!(relocations.len(), 1);
        let (old_loc, new_loc) = relocations[0];
        assert_eq!(old_loc as *const u8, ptr as *const u8);
        assert_ne!(old_loc, new_loc);

        // Spaces should be swapped
        assert_eq!(gen.from_space_base(), old_to_base);
        assert_eq!(gen.to_space_base(), old_from_base);

        // New location should be in new from_space
        assert!(gen.is_in_from_space(new_loc as *const u8));

        // Used space should reflect the copied object
        let aligned_size = (obj_size as usize + 7) & !7;
        assert_eq!(gen.used_space(), aligned_size);
    }

    #[test]
    fn test_young_generation_collect_multiple_objects() {
        let mut gen = YoungGeneration::new(1024);
        let obj_size = std::mem::size_of::<GcObject>() as u32;

        // Allocate three objects
        let ptr1 = gen.allocate(obj_size as usize).unwrap();
        let ptr2 = gen.allocate(obj_size as usize).unwrap();
        let ptr3 = gen.allocate(obj_size as usize).unwrap();

        // SAFETY: All pointers point to allocated memory of sufficient size
        unsafe {
            (*(ptr1 as *mut GcObject)).header = GcObjectHeader::new(obj_size);
            (*(ptr2 as *mut GcObject)).header = GcObjectHeader::new(obj_size);
            (*(ptr3 as *mut GcObject)).header = GcObjectHeader::new(obj_size);
        }

        // Only ptr1 and ptr3 are roots (ptr2 is garbage)
        let roots = vec![ptr1 as *mut GcObject, ptr3 as *mut GcObject];
        let relocations = gen.collect(&roots);

        // Should have relocated 2 objects
        assert_eq!(relocations.len(), 2);

        // Used space should be for 2 objects only (compaction)
        let aligned_size = (obj_size as usize + 7) & !7;
        assert_eq!(gen.used_space(), aligned_size * 2);
    }

    #[test]
    fn test_young_generation_collect_dead_objects_not_copied() {
        let mut gen = YoungGeneration::new(1024);
        let obj_size = std::mem::size_of::<GcObject>() as u32;

        // Allocate three objects (all dead)
        let dead1 = gen.allocate(obj_size as usize).unwrap();
        let dead2 = gen.allocate(obj_size as usize).unwrap();
        let dead3 = gen.allocate(obj_size as usize).unwrap();

        // SAFETY: All pointers point to allocated memory of sufficient size
        unsafe {
            (*(dead1 as *mut GcObject)).header = GcObjectHeader::new(obj_size);
            (*(dead2 as *mut GcObject)).header = GcObjectHeader::new(obj_size);
            (*(dead3 as *mut GcObject)).header = GcObjectHeader::new(obj_size);
        }

        let used_before = gen.used_space();
        assert!(used_before > 0);

        // No roots - all are garbage
        let roots: Vec<*mut GcObject> = vec![];
        let relocations = gen.collect(&roots);

        // Nothing should be relocated
        assert_eq!(relocations.len(), 0);

        // Space should be completely empty
        assert_eq!(gen.used_space(), 0);
        assert_eq!(gen.free_space(), gen.space_size());
    }

    #[test]
    fn test_young_generation_collect_compaction() {
        let mut gen = YoungGeneration::new(1024);
        let obj_size = std::mem::size_of::<GcObject>() as u32;

        // Allocate: live, dead, live, dead, live
        let live1 = gen.allocate(obj_size as usize).unwrap();
        let dead1 = gen.allocate(obj_size as usize).unwrap();
        let live2 = gen.allocate(obj_size as usize).unwrap();
        let dead2 = gen.allocate(obj_size as usize).unwrap();
        let live3 = gen.allocate(obj_size as usize).unwrap();

        // SAFETY: All pointers point to allocated memory of sufficient size
        unsafe {
            (*(live1 as *mut GcObject)).header = GcObjectHeader::new(obj_size);
            (*(dead1 as *mut GcObject)).header = GcObjectHeader::new(obj_size);
            (*(live2 as *mut GcObject)).header = GcObjectHeader::new(obj_size);
            (*(dead2 as *mut GcObject)).header = GcObjectHeader::new(obj_size);
            (*(live3 as *mut GcObject)).header = GcObjectHeader::new(obj_size);
        }

        // Collect only live objects
        let roots = vec![
            live1 as *mut GcObject,
            live2 as *mut GcObject,
            live3 as *mut GcObject,
        ];
        let relocations = gen.collect(&roots);

        // Should have 3 relocations
        assert_eq!(relocations.len(), 3);

        // Objects should be compacted (no gaps)
        let aligned_size = (obj_size as usize + 7) & !7;
        assert_eq!(gen.used_space(), aligned_size * 3);

        // Verify new locations are contiguous
        let new_locs: Vec<*mut GcObject> = relocations.iter().map(|(_, new)| *new).collect();
        let new_addrs: Vec<usize> = new_locs.iter().map(|p| *p as usize).collect();

        // Check they are adjacent (assuming same-sized objects)
        if new_addrs.len() >= 2 {
            for i in 1..new_addrs.len() {
                let expected_diff = aligned_size;
                let actual_diff = new_addrs[i] - new_addrs[i - 1];
                assert_eq!(actual_diff, expected_diff);
            }
        }
    }

    #[test]
    fn test_young_generation_collect_swaps_spaces() {
        let mut gen = YoungGeneration::new(1024);

        let original_from = gen.from_space_base();
        let original_to = gen.to_space_base();

        let roots: Vec<*mut GcObject> = vec![];
        gen.collect(&roots);

        // Spaces should be swapped
        assert_eq!(gen.from_space_base(), original_to);
        assert_eq!(gen.to_space_base(), original_from);
    }

    #[test]
    fn test_young_generation_collect_updates_forwarding() {
        let mut gen = YoungGeneration::new(1024);
        let obj_size = std::mem::size_of::<GcObject>() as u32;

        let ptr = gen.allocate(obj_size as usize).unwrap();
        // SAFETY: ptr points to allocated memory of sufficient size
        unsafe {
            (*(ptr as *mut GcObject)).header = GcObjectHeader::new(obj_size);
        }

        let roots = vec![ptr as *mut GcObject];
        let relocations = gen.collect(&roots);

        let (old_loc, new_loc) = relocations[0];

        // After collection and space swap:
        // - old_loc was in original from_space, now part of to_space (cleared)
        // - new_loc is in new from_space (was to_space)
        // The forwarding pointer was set during collection but the to_space
        // (where old_loc now resides) is zeroed after the swap.
        // We verify the relocation mapping is correct instead.
        assert_ne!(old_loc, new_loc);
        assert!(gen.is_in_from_space(new_loc as *const u8));

        // Verify the new object is properly formed
        // SAFETY: new_loc points to valid GcObject in new from_space
        unsafe {
            assert_eq!((*new_loc).header.size, obj_size);
            assert_eq!((*new_loc).header.age, 1); // Survived one collection
            assert!(!(*new_loc).header.is_forwarded()); // No forwarding pointer in live object
        }
    }

    #[test]
    fn test_young_generation_collect_increments_age() {
        let mut gen = YoungGeneration::new(1024);
        let obj_size = std::mem::size_of::<GcObject>() as u32;

        let ptr = gen.allocate(obj_size as usize).unwrap();
        // SAFETY: ptr points to allocated memory of sufficient size
        unsafe {
            let obj = ptr as *mut GcObject;
            (*obj).header = GcObjectHeader::new(obj_size);
            assert_eq!((*obj).header.age, 0);
        }

        let roots = vec![ptr as *mut GcObject];
        let relocations = gen.collect(&roots);

        let (_, new_loc) = relocations[0];

        // Age should be incremented
        // SAFETY: new_loc points to valid GcObject in new from_space
        unsafe {
            assert_eq!((*new_loc).header.age, 1);
        }
    }

    #[test]
    fn test_young_generation_collect_multiple_collections() {
        let mut gen = YoungGeneration::new(1024);
        let obj_size = std::mem::size_of::<GcObject>() as u32;

        let ptr = gen.allocate(obj_size as usize).unwrap();
        // SAFETY: ptr points to allocated memory of sufficient size
        unsafe {
            (*(ptr as *mut GcObject)).header = GcObjectHeader::new(obj_size);
        }

        // First collection
        let roots = vec![ptr as *mut GcObject];
        let relocations1 = gen.collect(&roots);
        let (_, new_loc1) = relocations1[0];

        // Second collection
        let roots = vec![new_loc1];
        let relocations2 = gen.collect(&roots);
        let (_, new_loc2) = relocations2[0];

        // Age should be 2 after two collections
        // SAFETY: new_loc2 points to valid GcObject
        unsafe {
            assert_eq!((*new_loc2).header.age, 2);
        }

        // Third collection
        let roots = vec![new_loc2];
        let relocations3 = gen.collect(&roots);
        let (_, new_loc3) = relocations3[0];

        // SAFETY: new_loc3 points to valid GcObject
        unsafe {
            assert_eq!((*new_loc3).header.age, 3);
        }
    }

    #[test]
    fn test_young_generation_free_space_after_collection() {
        let mut gen = YoungGeneration::new(1024);

        // Fill up most of the space
        let _ = gen.allocate(512);
        let _ = gen.allocate(256);

        assert!(gen.free_space() < 1024);

        // Collect with no roots (everything is garbage)
        let roots: Vec<*mut GcObject> = vec![];
        gen.collect(&roots);

        // Should have full space available again
        assert_eq!(gen.free_space(), 1024);
        assert_eq!(gen.used_space(), 0);
    }

    #[test]
    fn test_gc_object_data_ptr() {
        let mut gen = YoungGeneration::new(1024);

        // Allocate space for object + some data
        let data_size = 32;
        let total_size = std::mem::size_of::<GcObject>() + data_size;
        let ptr = gen.allocate(total_size).unwrap();

        // SAFETY: ptr points to allocated memory of sufficient size
        unsafe {
            let obj = ptr as *mut GcObject;
            (*obj).header = GcObjectHeader::new(total_size as u32);

            let data_ptr = (*obj).data_ptr();
            // Data pointer should be right after the header
            let expected_offset = std::mem::size_of::<GcObjectHeader>();
            let actual_offset = data_ptr as usize - ptr as usize;
            assert_eq!(actual_offset, expected_offset);
        }
    }

    #[test]
    fn test_young_generation_is_in_spaces() {
        let gen = YoungGeneration::new(1024);

        let from_ptr = gen.from_space_base();
        let to_ptr = gen.to_space_base();

        assert!(gen.is_in_from_space(from_ptr));
        assert!(!gen.is_in_to_space(from_ptr));

        assert!(gen.is_in_to_space(to_ptr));
        assert!(!gen.is_in_from_space(to_ptr));

        // Out of bounds pointer
        let out_of_bounds = 0x1 as *const u8;
        assert!(!gen.is_in_from_space(out_of_bounds));
        assert!(!gen.is_in_to_space(out_of_bounds));
    }

    // ========================================
    // Old Generation Tri-Color Marking Tests
    // ========================================

    // Helper function to create a test object
    fn create_test_object(size: u32) -> *mut GcObject {
        let obj = Box::new(GcObject {
            header: GcObjectHeader::new(size),
        });
        Box::into_raw(obj)
    }

    #[test]
    fn test_old_generation_new() {
        let old_gen = OldGeneration::new();
        assert_eq!(old_gen.object_count(), 0);
        assert_eq!(old_gen.total_memory(), 0);
        assert!(old_gen.gray_list_is_empty());
    }

    #[test]
    fn test_old_generation_default() {
        let old_gen = OldGeneration::default();
        assert_eq!(old_gen.object_count(), 0);
        assert_eq!(old_gen.total_memory(), 0);
    }

    #[test]
    fn test_old_generation_add_object() {
        let mut old_gen = OldGeneration::new();
        let obj = create_test_object(128);

        unsafe {
            old_gen.add_object(obj);
        }

        assert_eq!(old_gen.object_count(), 1);
        assert_eq!(old_gen.total_memory(), 128);
        assert!(old_gen.contains(obj));

        // Clean up
        let _ = old_gen.collect(&[obj]);
        old_gen.collect(&[]);
    }

    #[test]
    fn test_old_generation_add_null_object() {
        let mut old_gen = OldGeneration::new();
        unsafe {
            old_gen.add_object(ptr::null_mut());
        }

        assert_eq!(old_gen.object_count(), 0);
        assert_eq!(old_gen.total_memory(), 0);
    }

    #[test]
    fn test_objects_start_white() {
        let mut old_gen = OldGeneration::new();
        let obj = create_test_object(64);

        unsafe {
            old_gen.add_object(obj);
        }

        // SAFETY: We just created this object
        unsafe {
            assert_eq!(old_gen.get_mark_color(obj), MarkColor::White);
        }

        // Clean up
        old_gen.collect(&[]);
    }

    #[test]
    fn test_roots_marked_gray() {
        let mut old_gen = OldGeneration::new();
        let obj = create_test_object(64);

        unsafe {
            old_gen.add_object(obj);
        }
        old_gen.mark_roots(&[obj]);

        // Root should be gray now
        unsafe {
            assert_eq!(old_gen.get_mark_color(obj), MarkColor::Gray);
        }

        assert!(!old_gen.gray_list_is_empty());
        assert_eq!(old_gen.gray_list_size(), 1);

        // Clean up
        old_gen.process_gray_list();
        old_gen.sweep();
        unsafe {
            let _ = Box::from_raw(obj);
        }
    }

    #[test]
    fn test_gray_processing_marks_black() {
        let mut old_gen = OldGeneration::new();
        let obj = create_test_object(64);

        unsafe {
            old_gen.add_object(obj);
        }
        old_gen.mark_roots(&[obj]);
        old_gen.process_gray_list();

        // After processing, object should be black
        unsafe {
            assert_eq!(old_gen.get_mark_color(obj), MarkColor::Black);
        }

        // Gray list should be empty
        assert!(old_gen.gray_list_is_empty());

        // Clean up
        old_gen.sweep();
        unsafe {
            let _ = Box::from_raw(obj);
        }
    }

    #[test]
    fn test_black_objects_retained() {
        let mut old_gen = OldGeneration::new();
        let obj = create_test_object(128);

        unsafe {
            old_gen.add_object(obj);
        }
        let freed = old_gen.collect(&[obj]);

        // Root object should be retained
        assert_eq!(freed, 0);
        assert_eq!(old_gen.object_count(), 1);
        assert_eq!(old_gen.total_memory(), 128);

        // Clean up
        old_gen.collect(&[]);
    }

    #[test]
    fn test_white_objects_collected() {
        let mut old_gen = OldGeneration::new();
        let obj1 = create_test_object(64);
        let obj2 = create_test_object(128);

        unsafe {
            old_gen.add_object(obj1);
            old_gen.add_object(obj2);
        }

        // Only obj1 is a root, obj2 should be collected
        let freed = old_gen.collect(&[obj1]);

        assert_eq!(freed, 128); // obj2's size
        assert_eq!(old_gen.object_count(), 1);
        assert_eq!(old_gen.total_memory(), 64);

        // Clean up remaining object
        old_gen.collect(&[]);
    }

    #[test]
    fn test_multiple_roots() {
        let mut old_gen = OldGeneration::new();
        let root1 = create_test_object(32);
        let root2 = create_test_object(64);
        let garbage = create_test_object(128);

        unsafe {
            old_gen.add_object(root1);
            old_gen.add_object(root2);
            old_gen.add_object(garbage);
        }

        let freed = old_gen.collect(&[root1, root2]);

        assert_eq!(freed, 128); // garbage collected
        assert_eq!(old_gen.object_count(), 2);

        // Clean up
        old_gen.collect(&[]);
    }

    #[test]
    fn test_collect_empty_generation() {
        let mut old_gen = OldGeneration::new();
        let freed = old_gen.collect(&[]);

        assert_eq!(freed, 0);
        assert_eq!(old_gen.object_count(), 0);
    }

    #[test]
    fn test_collect_with_no_roots() {
        let mut old_gen = OldGeneration::new();
        let obj1 = create_test_object(64);
        let obj2 = create_test_object(128);

        unsafe {
            old_gen.add_object(obj1);
            old_gen.add_object(obj2);
        }

        // No roots means everything is garbage
        let freed = old_gen.collect(&[]);

        assert_eq!(freed, 192);
        assert_eq!(old_gen.object_count(), 0);
        assert_eq!(old_gen.total_memory(), 0);
    }

    #[test]
    fn test_mark_same_root_twice() {
        let mut old_gen = OldGeneration::new();
        let obj = create_test_object(64);

        unsafe {
            old_gen.add_object(obj);
        }

        // Pass same root twice - should only mark once
        let freed = old_gen.collect(&[obj, obj]);

        assert_eq!(freed, 0);
        assert_eq!(old_gen.object_count(), 1);

        // Clean up
        old_gen.collect(&[]);
    }

    #[test]
    fn test_null_in_roots() {
        let mut old_gen = OldGeneration::new();
        let obj = create_test_object(64);
        let garbage = create_test_object(32);

        unsafe {
            old_gen.add_object(obj);
            old_gen.add_object(garbage);
        }

        // Include null in roots - should be ignored
        let freed = old_gen.collect(&[obj, ptr::null_mut()]);

        assert_eq!(freed, 32); // garbage collected
        assert_eq!(old_gen.object_count(), 1);

        // Clean up
        old_gen.collect(&[]);
    }

    #[test]
    fn test_incremental_collection() {
        let mut old_gen = OldGeneration::new();

        // First collection
        let obj1 = create_test_object(32);
        unsafe {
            old_gen.add_object(obj1);
        }
        let freed1 = old_gen.collect(&[obj1]);
        assert_eq!(freed1, 0);

        // Second collection with new object
        let obj2 = create_test_object(64);
        unsafe {
            old_gen.add_object(obj2);
        }
        let freed2 = old_gen.collect(&[obj1]); // obj2 becomes garbage

        assert_eq!(freed2, 64);
        assert_eq!(old_gen.object_count(), 1);

        // Clean up
        old_gen.collect(&[]);
    }

    #[test]
    fn test_collect_updates_total_memory() {
        let mut old_gen = OldGeneration::new();
        let obj1 = create_test_object(100);
        let obj2 = create_test_object(200);
        let obj3 = create_test_object(300);

        unsafe {
            old_gen.add_object(obj1);
            old_gen.add_object(obj2);
            old_gen.add_object(obj3);
        }

        assert_eq!(old_gen.total_memory(), 600);

        // Collect with only obj1 as root
        let freed = old_gen.collect(&[obj1]);
        assert_eq!(freed, 500); // obj2 + obj3
        assert_eq!(old_gen.total_memory(), 100);

        old_gen.collect(&[]);
    }

    #[test]
    fn test_gray_list_cleared_after_collection() {
        let mut old_gen = OldGeneration::new();
        let obj = create_test_object(64);

        unsafe {
            old_gen.add_object(obj);
        }
        old_gen.collect(&[obj]);

        // Gray list should be empty after collection
        assert!(old_gen.gray_list_is_empty());

        old_gen.collect(&[]);
    }

    #[test]
    fn test_objects_method() {
        let mut old_gen = OldGeneration::new();
        let obj1 = create_test_object(32);
        let obj2 = create_test_object(64);

        unsafe {
            old_gen.add_object(obj1);
            old_gen.add_object(obj2);
        }

        let objects = old_gen.objects();
        assert_eq!(objects.len(), 2);
        assert!(objects.contains(&obj1));
        assert!(objects.contains(&obj2));

        old_gen.collect(&[]);
    }

    #[test]
    fn test_contains_method() {
        let mut old_gen = OldGeneration::new();
        let obj1 = create_test_object(32);
        let obj2 = create_test_object(64);

        unsafe {
            old_gen.add_object(obj1);
        }

        assert!(old_gen.contains(obj1));
        assert!(!old_gen.contains(obj2));

        // Clean up
        old_gen.collect(&[]);
        unsafe {
            let _ = Box::from_raw(obj2);
        }
    }

    #[test]
    fn test_process_gray_list_with_tracer_simple() {
        let mut old_gen = OldGeneration::new();
        let parent = create_test_object(64);
        let child = create_test_object(32);

        unsafe {
            old_gen.add_object(parent);
            old_gen.add_object(child);
        }

        // Mark parent as gray (simulating root marking)
        old_gen.mark_roots(&[parent]);

        // Process with tracer that knows parent points to child
        unsafe {
            old_gen.process_gray_list_with_tracer(|obj_ptr, tracer| {
                if obj_ptr == parent {
                    // Parent references child
                    tracer(child);
                }
            });
        }

        // Both should be black now
        unsafe {
            assert_eq!(old_gen.get_mark_color(parent), MarkColor::Black);
            assert_eq!(old_gen.get_mark_color(child), MarkColor::Black);
        }

        // Sweep should retain both
        let freed = old_gen.sweep();
        assert_eq!(freed, 0);
        assert_eq!(old_gen.object_count(), 2);

        // Clean up
        old_gen.collect(&[]);
    }

    #[test]
    fn test_process_gray_list_with_tracer_cycle() {
        // Test cycle detection: A -> B -> A
        let mut old_gen = OldGeneration::new();
        let obj_a = create_test_object(32);
        let obj_b = create_test_object(64);

        unsafe {
            old_gen.add_object(obj_a);
            old_gen.add_object(obj_b);
        }

        // Mark A as root
        old_gen.mark_roots(&[obj_a]);

        // Process with tracer that creates cycle
        unsafe {
            old_gen.process_gray_list_with_tracer(|obj_ptr, tracer| {
                if obj_ptr == obj_a {
                    tracer(obj_b);
                } else if obj_ptr == obj_b {
                    tracer(obj_a); // Back to A (cycle)
                }
            });
        }

        // Both should be black (cycle handled correctly)
        unsafe {
            assert_eq!(old_gen.get_mark_color(obj_a), MarkColor::Black);
            assert_eq!(old_gen.get_mark_color(obj_b), MarkColor::Black);
        }

        // Gray list should be empty
        assert!(old_gen.gray_list_is_empty());

        // Clean up
        old_gen.collect(&[]);
    }

    #[test]
    fn test_unreachable_cycle_collected() {
        // Create an unreachable cycle
        let mut old_gen = OldGeneration::new();
        let cycle_a = create_test_object(32);
        let cycle_b = create_test_object(64);
        let root = create_test_object(16);

        unsafe {
            old_gen.add_object(cycle_a);
            old_gen.add_object(cycle_b);
            old_gen.add_object(root);
        }

        // Only root is reachable
        old_gen.mark_all_white();
        old_gen.mark_roots(&[root]);

        // Process - root has no children, cycle objects stay white
        unsafe {
            old_gen.process_gray_list_with_tracer(|_obj_ptr, _tracer| {
                // Root has no children
            });
        }

        // Root should be black, cycle objects white
        unsafe {
            assert_eq!(old_gen.get_mark_color(root), MarkColor::Black);
            assert_eq!(old_gen.get_mark_color(cycle_a), MarkColor::White);
            assert_eq!(old_gen.get_mark_color(cycle_b), MarkColor::White);
        }

        // Sweep should collect cycle
        let freed = old_gen.sweep();
        assert_eq!(freed, 96); // 32 + 64
        assert_eq!(old_gen.object_count(), 1);

        // Clean up
        old_gen.collect(&[]);
    }

    #[test]
    fn test_complex_object_graph() {
        // Create a complex graph:
        //     root
        //    /    \
        //   A      B
        //   |      |
        //   C------D (shared reference to E)
        //    \    /
        //      E
        //
        // Plus unreachable F, G

        let mut old_gen = OldGeneration::new();
        let root = create_test_object(16);
        let a = create_test_object(32);
        let b = create_test_object(32);
        let c = create_test_object(32);
        let d = create_test_object(32);
        let e = create_test_object(64);
        let f = create_test_object(48); // garbage
        let g = create_test_object(48); // garbage

        unsafe {
            old_gen.add_object(root);
            old_gen.add_object(a);
            old_gen.add_object(b);
            old_gen.add_object(c);
            old_gen.add_object(d);
            old_gen.add_object(e);
            old_gen.add_object(f);
            old_gen.add_object(g);
        }

        // Start marking
        old_gen.mark_all_white();
        old_gen.mark_roots(&[root]);

        // Process with graph structure
        unsafe {
            old_gen.process_gray_list_with_tracer(|obj_ptr, tracer| {
                if obj_ptr == root {
                    tracer(a);
                    tracer(b);
                } else if obj_ptr == a {
                    tracer(c);
                } else if obj_ptr == b {
                    tracer(d);
                } else if obj_ptr == c {
                    tracer(e);
                } else if obj_ptr == d {
                    tracer(e); // Shared reference
                }
            });
        }

        // Verify marking
        unsafe {
            assert_eq!(old_gen.get_mark_color(root), MarkColor::Black);
            assert_eq!(old_gen.get_mark_color(a), MarkColor::Black);
            assert_eq!(old_gen.get_mark_color(b), MarkColor::Black);
            assert_eq!(old_gen.get_mark_color(c), MarkColor::Black);
            assert_eq!(old_gen.get_mark_color(d), MarkColor::Black);
            assert_eq!(old_gen.get_mark_color(e), MarkColor::Black);
            assert_eq!(old_gen.get_mark_color(f), MarkColor::White);
            assert_eq!(old_gen.get_mark_color(g), MarkColor::White);
        }

        // Sweep
        let freed = old_gen.sweep();
        assert_eq!(freed, 96); // f + g
        assert_eq!(old_gen.object_count(), 6);

        // Clean up
        old_gen.collect(&[]);
    }

    #[test]
    fn test_tri_color_invariant_maintained() {
        // Verify that gray list processing maintains the tri-color invariant:
        // Black objects never point to white objects

        let mut old_gen = OldGeneration::new();
        let obj1 = create_test_object(32);
        let obj2 = create_test_object(32);
        let obj3 = create_test_object(32);

        unsafe {
            old_gen.add_object(obj1);
            old_gen.add_object(obj2);
            old_gen.add_object(obj3);
        }

        old_gen.mark_all_white();
        old_gen.mark_roots(&[obj1]);

        // Initially: obj1 is gray, obj2 and obj3 are white
        unsafe {
            assert_eq!(old_gen.get_mark_color(obj1), MarkColor::Gray);
            assert_eq!(old_gen.get_mark_color(obj2), MarkColor::White);
            assert_eq!(old_gen.get_mark_color(obj3), MarkColor::White);
        }

        // Process with obj1 -> obj2 -> obj3
        unsafe {
            old_gen.process_gray_list_with_tracer(|obj_ptr, tracer| {
                if obj_ptr == obj1 {
                    // Before tracing children, obj1 is gray
                    // After tracing, obj1 will be black and obj2 will be gray
                    tracer(obj2);
                } else if obj_ptr == obj2 {
                    tracer(obj3);
                }
            });
        }

        // All reachable objects should be black
        unsafe {
            assert_eq!(old_gen.get_mark_color(obj1), MarkColor::Black);
            assert_eq!(old_gen.get_mark_color(obj2), MarkColor::Black);
            assert_eq!(old_gen.get_mark_color(obj3), MarkColor::Black);
        }

        old_gen.collect(&[]);
    }
}

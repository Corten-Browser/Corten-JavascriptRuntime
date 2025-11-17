//! Write barrier implementation for generational GC
//!
//! Write barriers track when old generation objects reference young generation
//! objects, maintaining the remembered set for efficient garbage collection.
//!
//! When an old generation object stores a reference to a young generation object,
//! we must record this to avoid collecting the young object prematurely. This is
//! called a "remembered set".

use crate::gc::{GcObject, OldGeneration, YoungGeneration};
use core_types::Value;
use std::collections::HashSet;
use std::sync::Mutex;

/// Object type placeholder (to be replaced with actual Object type)
pub struct Object;

// Re-export for convenience
pub use crate::gc::{OldGeneration as OldGen, YoungGeneration as YoungGen};

/// Global remembered set tracking old-to-young references.
///
/// When an old generation object is modified to point to a young generation
/// object, the old object's address is recorded here. During young GC,
/// these objects are treated as additional roots.
///
/// This implementation uses Mutex for thread-safe access.
pub struct RememberedSet {
    /// Set of old generation object addresses that contain pointers to young gen
    cards: Mutex<HashSet<*mut GcObject>>,
}

// SAFETY: RememberedSet uses internal Mutex for synchronization
unsafe impl Send for RememberedSet {}
unsafe impl Sync for RememberedSet {}

impl Default for RememberedSet {
    fn default() -> Self {
        Self::new()
    }
}

impl RememberedSet {
    /// Creates an empty remembered set.
    pub fn new() -> Self {
        RememberedSet {
            cards: Mutex::new(HashSet::new()),
        }
    }

    /// Adds an object to the remembered set.
    ///
    /// # Arguments
    ///
    /// * `old_obj` - Pointer to old generation object that references young gen
    pub fn add(&self, old_obj: *mut GcObject) {
        if old_obj.is_null() {
            return;
        }
        let mut cards = self.cards.lock().unwrap();
        cards.insert(old_obj);
    }

    /// Removes an object from the remembered set.
    ///
    /// Called when the old object no longer references young gen objects.
    pub fn remove(&self, old_obj: *mut GcObject) {
        let mut cards = self.cards.lock().unwrap();
        cards.remove(&old_obj);
    }

    /// Clears all entries from the remembered set.
    ///
    /// Called after a young generation collection when all references
    /// have been updated or promoted.
    pub fn clear(&self) {
        let mut cards = self.cards.lock().unwrap();
        cards.clear();
    }

    /// Returns all objects in the remembered set as GC roots.
    ///
    /// These objects should be scanned during young generation collection.
    pub fn get_roots(&self) -> Vec<*mut GcObject> {
        let cards = self.cards.lock().unwrap();
        cards.iter().copied().collect()
    }

    /// Returns the number of entries in the remembered set.
    pub fn size(&self) -> usize {
        let cards = self.cards.lock().unwrap();
        cards.len()
    }

    /// Returns true if the remembered set is empty.
    pub fn is_empty(&self) -> bool {
        let cards = self.cards.lock().unwrap();
        cards.is_empty()
    }

    /// Checks if an object is in the remembered set.
    pub fn contains(&self, obj: *mut GcObject) -> bool {
        let cards = self.cards.lock().unwrap();
        cards.contains(&obj)
    }

    /// Returns an iterator over all remembered objects (snapshot).
    pub fn iter(&self) -> impl Iterator<Item = *mut GcObject> {
        // Return a snapshot since we can't hold the lock
        self.get_roots().into_iter()
    }

    /// Returns the number of entries (alias for size).
    pub fn len(&self) -> usize {
        self.size()
    }

    /// Returns all entries as a vector (alias for get_roots).
    pub fn as_roots(&self) -> Vec<*mut GcObject> {
        self.get_roots()
    }
}

/// Card table for efficient write barrier checking.
///
/// Divides the heap into fixed-size cards (typically 512 bytes).
/// Each card is marked dirty when any reference in that region is modified.
/// This allows faster scanning than tracking individual references.
#[derive(Debug)]
pub struct CardTable {
    /// Table of dirty bits, one per card
    cards: Vec<bool>,
    /// Size of each card in bytes
    card_size: usize,
    /// Base address of the heap region covered
    base_address: usize,
    /// Total size of the heap region covered
    heap_size: usize,
}

impl CardTable {
    /// Creates a new card table for a heap region.
    ///
    /// # Arguments
    ///
    /// * `base_address` - Starting address of the heap region
    /// * `heap_size` - Total size of the heap region in bytes
    /// * `card_size` - Size of each card (default: 512 bytes)
    pub fn new(base_address: usize, heap_size: usize, card_size: usize) -> Self {
        let num_cards = (heap_size + card_size - 1) / card_size;
        CardTable {
            cards: vec![false; num_cards],
            card_size,
            base_address,
            heap_size,
        }
    }

    /// Creates a card table with default 512-byte cards.
    pub fn with_default_card_size(base_address: usize, heap_size: usize) -> Self {
        Self::new(base_address, heap_size, 512)
    }

    /// Marks the card containing the given address as dirty.
    ///
    /// # Arguments
    ///
    /// * `address` - Address of the modified reference
    ///
    /// # Returns
    ///
    /// True if the address was within the card table's range.
    pub fn mark_dirty(&mut self, address: usize) -> bool {
        if address < self.base_address || address >= self.base_address + self.heap_size {
            return false;
        }

        let card_index = (address - self.base_address) / self.card_size;
        if card_index < self.cards.len() {
            self.cards[card_index] = true;
            true
        } else {
            false
        }
    }

    /// Clears the dirty bit for a specific card.
    pub fn clear_card(&mut self, card_index: usize) {
        if card_index < self.cards.len() {
            self.cards[card_index] = false;
        }
    }

    /// Clears all dirty bits.
    pub fn clear_all(&mut self) {
        self.cards.iter_mut().for_each(|card| *card = false);
    }

    /// Returns true if the card at the given index is dirty.
    pub fn is_dirty(&self, card_index: usize) -> bool {
        card_index < self.cards.len() && self.cards[card_index]
    }

    /// Returns indices of all dirty cards.
    pub fn dirty_cards(&self) -> Vec<usize> {
        self.cards
            .iter()
            .enumerate()
            .filter_map(|(idx, &is_dirty)| if is_dirty { Some(idx) } else { None })
            .collect()
    }

    /// Returns the address range covered by a card.
    pub fn card_range(&self, card_index: usize) -> Option<(usize, usize)> {
        if card_index >= self.cards.len() {
            return None;
        }

        let start = self.base_address + (card_index * self.card_size);
        let end = (start + self.card_size).min(self.base_address + self.heap_size);
        Some((start, end))
    }

    /// Returns the card index for a given address.
    pub fn address_to_card(&self, address: usize) -> Option<usize> {
        if address < self.base_address || address >= self.base_address + self.heap_size {
            return None;
        }
        Some((address - self.base_address) / self.card_size)
    }

    /// Returns the number of cards in the table.
    pub fn num_cards(&self) -> usize {
        self.cards.len()
    }

    /// Returns the number of dirty cards.
    pub fn num_dirty_cards(&self) -> usize {
        self.cards.iter().filter(|&&dirty| dirty).count()
    }

    /// Returns the card size.
    pub fn card_size(&self) -> usize {
        self.card_size
    }

    /// Returns indices of all dirty cards (alias for dirty_cards).
    pub fn get_dirty_cards(&self) -> Vec<usize> {
        self.dirty_cards()
    }

    /// Clears all dirty bits (alias for clear_all).
    pub fn clear(&mut self) {
        self.clear_all()
    }
}

/// Write barrier implementation for generational garbage collection.
///
/// This function should be called whenever a reference field is updated.
/// It checks if the write creates an old-to-young reference and records
/// it in the remembered set if so.
///
/// # Safety
///
/// The caller must ensure:
/// - `obj` is a valid pointer to a `GcObject` or null
/// - `slot` is a valid pointer to a mutable location containing a `*mut GcObject`
/// - `new_val` is either null or a valid pointer to a `GcObject`
/// - All references (remembered_set, young_gen, old_gen) are valid
/// - The caller has exclusive access to the slot being written
///
/// # Arguments
///
/// * `obj` - Pointer to the object being written to (the container)
/// * `slot` - Pointer to the slot being updated (the field)
/// * `new_val` - The new value being stored (the reference)
/// * `remembered_set` - The remembered set for tracking old-to-young references
/// * `young_gen` - Reference to the young generation
/// * `old_gen` - Reference to the old generation
pub unsafe fn write_barrier_gc(
    obj: *mut GcObject,
    slot: *mut *mut GcObject,
    new_val: *mut GcObject,
    remembered_set: &RememberedSet,
    young_gen: &YoungGeneration,
    old_gen: &OldGeneration,
) {
    // SAFETY: Caller guarantees slot is a valid pointer to writable memory.
    // We perform the write operation first.
    *slot = new_val;

    // Skip if the container object is null
    if obj.is_null() {
        return;
    }

    // Skip if the new value is null (no reference to track)
    if new_val.is_null() {
        return;
    }

    // Check if this is an old-to-young reference:
    // Container object is in old generation AND new value is in young generation
    let obj_in_old = old_gen.contains(obj);
    let new_val_in_young = young_gen.is_in_from_space(new_val as *const u8);

    if obj_in_old && new_val_in_young {
        // Old generation object now references young generation object.
        // Add to remembered set so this old object is treated as a root
        // during young generation collection.
        remembered_set.add(obj);
    }
}

/// Write barrier for maintaining the remembered set
///
/// # Safety
/// - `obj` must be a valid pointer to a heap-allocated object
/// - `slot` must be a valid pointer within the object
/// - The caller must ensure proper synchronization in multi-threaded contexts
///
/// This function must be called whenever a reference is written to maintain
/// the generational GC invariant. If an old generation object is modified
/// to point to a young generation object, it must be recorded in the
/// remembered set.
pub unsafe fn write_barrier(_obj: *mut Object, _slot: *mut Value, _new_val: Value) {
    // The actual write barrier logic is handled by the Heap struct,
    // which has access to both generations and the remembered set.
    // This function serves as a placeholder for the public API.
    // In practice, the Heap's write operation will check generations
    // and update the remembered set accordingly.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remembered_set_new() {
        let rs = RememberedSet::new();
        assert!(rs.is_empty());
        assert_eq!(rs.len(), 0);
    }

    #[test]
    fn test_remembered_set_add() {
        let rs = RememberedSet::new();
        let fake_ptr = 0x1000 as *mut GcObject;

        rs.add(fake_ptr);
        assert_eq!(rs.len(), 1);
        assert!(rs.contains(fake_ptr));
    }

    #[test]
    fn test_remembered_set_add_duplicate() {
        let rs = RememberedSet::new();
        let fake_ptr = 0x1000 as *mut GcObject;

        rs.add(fake_ptr);
        rs.add(fake_ptr); // Adding same pointer again
        assert_eq!(rs.len(), 1); // Should still be 1 (HashSet)
    }

    #[test]
    fn test_remembered_set_remove() {
        let rs = RememberedSet::new();
        let ptr1 = 0x1000 as *mut GcObject;
        let ptr2 = 0x2000 as *mut GcObject;

        rs.add(ptr1);
        rs.add(ptr2);
        assert_eq!(rs.len(), 2);

        rs.remove(ptr1);
        assert_eq!(rs.len(), 1);
        assert!(!rs.contains(ptr1));
        assert!(rs.contains(ptr2));
    }

    #[test]
    fn test_remembered_set_clear() {
        let rs = RememberedSet::new();
        rs.add(0x1000 as *mut GcObject);
        rs.add(0x2000 as *mut GcObject);

        rs.clear();
        assert!(rs.is_empty());
        assert_eq!(rs.len(), 0);
    }

    #[test]
    fn test_remembered_set_as_roots() {
        let rs = RememberedSet::new();
        let ptr1 = 0x1000 as *mut GcObject;
        let ptr2 = 0x2000 as *mut GcObject;

        rs.add(ptr1);
        rs.add(ptr2);

        let roots = rs.as_roots();
        assert_eq!(roots.len(), 2);
        assert!(roots.contains(&ptr1));
        assert!(roots.contains(&ptr2));
    }

    #[test]
    fn test_card_table_new() {
        let ct = CardTable::new(0x1000, 4096, 512);
        assert_eq!(ct.num_cards(), 8); // 4096 / 512 = 8
        assert_eq!(ct.card_size(), 512);
        assert_eq!(ct.num_dirty_cards(), 0);
    }

    #[test]
    fn test_card_table_with_default() {
        let ct = CardTable::with_default_card_size(0x1000, 2048);
        assert_eq!(ct.card_size(), 512);
        assert_eq!(ct.num_cards(), 4); // 2048 / 512 = 4
    }

    #[test]
    fn test_card_table_mark_dirty() {
        let mut ct = CardTable::new(0x1000, 4096, 512);

        // Mark first card dirty
        assert!(ct.mark_dirty(0x1000));
        assert!(ct.is_dirty(0));

        // Mark third card dirty
        assert!(ct.mark_dirty(0x1400)); // 0x1000 + 1024 = third card
        assert!(ct.is_dirty(2));

        assert_eq!(ct.num_dirty_cards(), 2);
    }

    #[test]
    fn test_card_table_mark_dirty_out_of_range() {
        let mut ct = CardTable::new(0x1000, 4096, 512);

        // Before range
        assert!(!ct.mark_dirty(0x500));

        // After range
        assert!(!ct.mark_dirty(0x3000));

        assert_eq!(ct.num_dirty_cards(), 0);
    }

    #[test]
    fn test_card_table_clear_card() {
        let mut ct = CardTable::new(0x1000, 4096, 512);

        ct.mark_dirty(0x1000);
        ct.mark_dirty(0x1200);
        assert_eq!(ct.num_dirty_cards(), 2);

        ct.clear_card(0);
        assert!(!ct.is_dirty(0));
        assert!(ct.is_dirty(1));
        assert_eq!(ct.num_dirty_cards(), 1);
    }

    #[test]
    fn test_card_table_clear_all() {
        let mut ct = CardTable::new(0x1000, 4096, 512);

        ct.mark_dirty(0x1000);
        ct.mark_dirty(0x1200);
        ct.mark_dirty(0x1800);

        ct.clear_all();
        assert_eq!(ct.num_dirty_cards(), 0);
    }

    #[test]
    fn test_card_table_dirty_cards() {
        let mut ct = CardTable::new(0x1000, 4096, 512);

        ct.mark_dirty(0x1000); // Card 0
        ct.mark_dirty(0x1400); // Card 2
        ct.mark_dirty(0x1E00); // Card 7

        let dirty = ct.dirty_cards();
        assert_eq!(dirty.len(), 3);
        assert!(dirty.contains(&0));
        assert!(dirty.contains(&2));
        assert!(dirty.contains(&7));
    }

    #[test]
    fn test_card_table_card_range() {
        let ct = CardTable::new(0x1000, 4096, 512);

        let range = ct.card_range(0).unwrap();
        assert_eq!(range, (0x1000, 0x1200));

        let range = ct.card_range(2).unwrap();
        assert_eq!(range, (0x1400, 0x1600));

        let range = ct.card_range(7).unwrap();
        assert_eq!(range, (0x1E00, 0x2000));

        // Out of range
        assert!(ct.card_range(8).is_none());
    }

    #[test]
    fn test_card_table_address_to_card() {
        let ct = CardTable::new(0x1000, 4096, 512);

        assert_eq!(ct.address_to_card(0x1000), Some(0));
        assert_eq!(ct.address_to_card(0x11FF), Some(0));
        assert_eq!(ct.address_to_card(0x1200), Some(1));
        assert_eq!(ct.address_to_card(0x1FFF), Some(7));

        // Out of range
        assert_eq!(ct.address_to_card(0x500), None);
        assert_eq!(ct.address_to_card(0x3000), None);
    }

    #[test]
    fn test_card_table_uneven_size() {
        // Heap size not evenly divisible by card size
        let ct = CardTable::new(0x1000, 1500, 512);
        assert_eq!(ct.num_cards(), 3); // ceil(1500/512) = 3

        // Last card range should be truncated
        let range = ct.card_range(2).unwrap();
        assert_eq!(range, (0x1400, 0x15DC)); // 0x1000 + 1500 = 0x15DC
    }

    #[test]
    fn test_remembered_set_iter() {
        let rs = RememberedSet::new();
        let ptr1 = 0x1000 as *mut GcObject;
        let ptr2 = 0x2000 as *mut GcObject;

        rs.add(ptr1);
        rs.add(ptr2);

        let mut count = 0;
        for _ptr in rs.iter() {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_remembered_set_size() {
        let rs = RememberedSet::new();
        assert_eq!(rs.size(), 0);

        rs.add(0x1000 as *mut GcObject);
        assert_eq!(rs.size(), 1);

        rs.add(0x2000 as *mut GcObject);
        assert_eq!(rs.size(), 2);
    }

    #[test]
    fn test_remembered_set_get_roots() {
        let rs = RememberedSet::new();
        let ptr1 = 0x1000 as *mut GcObject;
        let ptr2 = 0x2000 as *mut GcObject;

        rs.add(ptr1);
        rs.add(ptr2);

        let roots = rs.get_roots();
        assert_eq!(roots.len(), 2);
        assert!(roots.contains(&ptr1));
        assert!(roots.contains(&ptr2));
    }

    #[test]
    fn test_remembered_set_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let rs = Arc::new(RememberedSet::new());
        let mut handles = vec![];

        // Spawn multiple threads adding to remembered set
        for i in 0..4 {
            let rs_clone = Arc::clone(&rs);
            let handle = thread::spawn(move || {
                let ptr = (0x1000 + i * 0x100) as *mut GcObject;
                rs_clone.add(ptr);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(rs.size(), 4);
    }

    #[test]
    fn test_card_table_get_dirty_cards() {
        let mut ct = CardTable::new(0x1000, 4096, 512);

        ct.mark_dirty(0x1000); // Card 0
        ct.mark_dirty(0x1400); // Card 2

        let dirty = ct.get_dirty_cards();
        assert_eq!(dirty.len(), 2);
        assert!(dirty.contains(&0));
        assert!(dirty.contains(&2));
    }

    #[test]
    fn test_card_table_clear() {
        let mut ct = CardTable::new(0x1000, 4096, 512);

        ct.mark_dirty(0x1000);
        ct.mark_dirty(0x1200);
        assert_eq!(ct.num_dirty_cards(), 2);

        ct.clear();
        assert_eq!(ct.num_dirty_cards(), 0);
    }

    #[test]
    fn test_write_barrier_gc_null_object() {
        use crate::gc::{GcObjectHeader, OldGeneration, YoungGeneration};
        use std::ptr;

        let rs = RememberedSet::new();
        let young_gen = YoungGeneration::new(1024);
        let old_gen = OldGeneration::new();

        let mut slot: *mut GcObject = ptr::null_mut();
        let new_val = 0x2000 as *mut GcObject;

        unsafe {
            write_barrier_gc(ptr::null_mut(), &mut slot, new_val, &rs, &young_gen, &old_gen);
        }

        // Slot should be updated
        assert_eq!(slot, new_val);
        // But nothing added to remembered set (obj is null)
        assert_eq!(rs.size(), 0);
    }

    #[test]
    fn test_write_barrier_gc_null_value() {
        use crate::gc::{GcObjectHeader, OldGeneration, YoungGeneration};
        use std::ptr;

        let rs = RememberedSet::new();
        let young_gen = YoungGeneration::new(1024);
        let mut old_gen = OldGeneration::new();

        let old_obj = Box::into_raw(Box::new(crate::gc::GcObject {
            header: GcObjectHeader::new(64),
        }));

        unsafe {
            old_gen.add_object(old_obj);
        }

        let mut slot: *mut GcObject = 0x2000 as *mut GcObject;

        unsafe {
            write_barrier_gc(old_obj, &mut slot, ptr::null_mut(), &rs, &young_gen, &old_gen);
        }

        // Slot should be updated to null
        assert!(slot.is_null());
        // Nothing added to remembered set (new_val is null)
        assert_eq!(rs.size(), 0);

        // Clean up
        old_gen.collect(&[]);
    }

    #[test]
    fn test_write_barrier_gc_detects_old_to_young() {
        use crate::gc::{GcObjectHeader, OldGeneration, YoungGeneration};

        let rs = RememberedSet::new();
        let mut young_gen = YoungGeneration::new(1024);
        let mut old_gen = OldGeneration::new();

        // Create old generation object
        let old_obj = Box::into_raw(Box::new(crate::gc::GcObject {
            header: GcObjectHeader::new(64),
        }));
        unsafe {
            old_gen.add_object(old_obj);
        }

        // Create young generation object
        let young_obj_ptr = young_gen.allocate(32).unwrap() as *mut GcObject;
        unsafe {
            (*young_obj_ptr).header = GcObjectHeader::new(32);
        }

        let mut slot: *mut GcObject = std::ptr::null_mut();

        // Perform write barrier: old object stores reference to young object
        unsafe {
            write_barrier_gc(old_obj, &mut slot, young_obj_ptr, &rs, &young_gen, &old_gen);
        }

        // Should detect old-to-young reference
        assert_eq!(rs.size(), 1);
        assert!(rs.contains(old_obj));

        old_gen.collect(&[]);
    }

    #[test]
    fn test_write_barrier_gc_no_old_to_old() {
        use crate::gc::{GcObjectHeader, OldGeneration, YoungGeneration};

        let rs = RememberedSet::new();
        let young_gen = YoungGeneration::new(1024);
        let mut old_gen = OldGeneration::new();

        // Both objects in old generation
        let old_obj1 = Box::into_raw(Box::new(crate::gc::GcObject {
            header: GcObjectHeader::new(64),
        }));
        let old_obj2 = Box::into_raw(Box::new(crate::gc::GcObject {
            header: GcObjectHeader::new(32),
        }));
        unsafe {
            old_gen.add_object(old_obj1);
            old_gen.add_object(old_obj2);
        }

        let mut slot: *mut GcObject = std::ptr::null_mut();

        unsafe {
            write_barrier_gc(old_obj1, &mut slot, old_obj2, &rs, &young_gen, &old_gen);
        }

        // Old-to-old reference: should NOT be in remembered set
        assert_eq!(rs.size(), 0);

        old_gen.collect(&[]);
    }

    #[test]
    fn test_write_barrier_gc_integration_with_young_gc() {
        use crate::gc::{GcObjectHeader, OldGeneration, YoungGeneration};

        let rs = RememberedSet::new();
        let mut young_gen = YoungGeneration::new(2048);
        let mut old_gen = OldGeneration::new();

        // Create old object that will reference young object
        let old_obj = Box::into_raw(Box::new(crate::gc::GcObject {
            header: GcObjectHeader::new(64),
        }));
        unsafe {
            old_gen.add_object(old_obj);
        }

        // Create young object
        let young_obj_ptr = young_gen.allocate(32).unwrap() as *mut GcObject;
        unsafe {
            (*young_obj_ptr).header = GcObjectHeader::new(32);
        }

        // Establish old-to-young reference
        let mut slot: *mut GcObject = std::ptr::null_mut();
        unsafe {
            write_barrier_gc(old_obj, &mut slot, young_obj_ptr, &rs, &young_gen, &old_gen);
        }

        // Get roots from remembered set
        let remembered_roots = rs.get_roots();
        assert_eq!(remembered_roots.len(), 1);
        assert_eq!(remembered_roots[0], old_obj);

        // During young GC, these roots would be used to find young objects
        // referenced from old generation

        old_gen.collect(&[]);
    }

    #[test]
    fn test_write_barrier_gc_clear_after_collection() {
        use crate::gc::{GcObjectHeader, OldGeneration, YoungGeneration};

        let rs = RememberedSet::new();
        let mut young_gen = YoungGeneration::new(1024);
        let mut old_gen = OldGeneration::new();

        let old_obj = Box::into_raw(Box::new(crate::gc::GcObject {
            header: GcObjectHeader::new(64),
        }));
        unsafe {
            old_gen.add_object(old_obj);
        }

        let young_obj_ptr = young_gen.allocate(32).unwrap() as *mut GcObject;
        unsafe {
            (*young_obj_ptr).header = GcObjectHeader::new(32);
        }

        let mut slot: *mut GcObject = std::ptr::null_mut();

        unsafe {
            write_barrier_gc(old_obj, &mut slot, young_obj_ptr, &rs, &young_gen, &old_gen);
        }

        assert_eq!(rs.size(), 1);

        // Simulate GC completion - clear remembered set
        rs.clear();
        assert_eq!(rs.size(), 0);

        old_gen.collect(&[]);
    }

    #[test]
    fn test_remembered_set_add_null() {
        let rs = RememberedSet::new();
        rs.add(std::ptr::null_mut());
        assert_eq!(rs.size(), 0);
    }
}


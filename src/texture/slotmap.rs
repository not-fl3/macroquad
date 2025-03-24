// Heavily reduced version of the `slotmap` crate: https://github.com/orlp/slotmap
use miniquad::TextureId;

pub(crate) type TextureSlotId = u32;

// Storage inside a slot or metadata for the freelist when vacant.
union SlotUnion {
    value: TextureId,
    next_free: u32,
}

/// Slot map, storage with stable unique keys.
pub(crate) struct TextureIdSlotMap {
    slots: Vec<SlotUnion>,
    free_head: u32,
    num_elems: u32,
}

impl TextureIdSlotMap {
    /// Constructs a new, empty [`TextureIdSlotMap`].
    pub fn new() -> Self {
        let slots = vec![SlotUnion { next_free: 0 }];

        Self {
            slots,
            free_head: 1,
            num_elems: 0,
        }
    }

    /// Returns the number of elements in the slot map.
    pub const fn len(&self) -> usize {
        self.num_elems as usize
    }

    /// Returns [`true`] if the slot map contains `key`.
    #[inline(always)]
    fn contains_key(&self, key: TextureSlotId) -> bool {
        self.slots.get(key as usize).is_some()
    }

    /// Inserts a value into the slot map. Returns a unique key that can be used
    /// to access this value.
    ///
    /// # Panics
    ///
    /// Panics if the number of elements in the slot map equals
    /// 2<sup>32</sup> - 2.
    pub fn insert(&mut self, value: TextureId) -> TextureSlotId {
        let new_num_elems = self.num_elems + 1;
        if new_num_elems == u32::MAX {
            panic!("SlotMap number of elements overflow");
        }

        if let Some(slot) = self.slots.get_mut(self.free_head as usize) {
            let kd = self.free_head;

            // Update.
            unsafe {
                self.free_head = slot.next_free;
                slot.value = value;
            }
            self.num_elems = new_num_elems;
            return kd;
        }

        let kd = self.slots.len() as u32;

        // Create new slot before adjusting freelist in case f or the allocation panics or errors.
        self.slots.push(SlotUnion { value });

        self.free_head = kd + 1;
        self.num_elems = new_num_elems;
        kd
    }

    /// Removes a key from the slot map if it is present.
    pub fn remove(&mut self, key: TextureSlotId) {
        if self.contains_key(key) {
            let idx = key as usize;

            // This is safe because we know that the slot is occupied.
            let slot = unsafe { self.slots.get_unchecked_mut(idx) };

            // Maintain freelist.
            slot.next_free = self.free_head;
            self.free_head = idx as u32;
            self.num_elems -= 1;
        }
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: TextureSlotId) -> Option<TextureId> {
        self.slots
            .get(key as usize)
            .map(|slot| unsafe { slot.value })
    }
}

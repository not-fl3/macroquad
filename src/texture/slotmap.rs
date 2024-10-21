// Heavily reduced version of the `slotmap` crate: https://github.com/orlp/slotmap
use miniquad::TextureId;
use std::fmt;
use std::num::NonZeroU32;

#[derive(Copy, Clone, PartialEq)]
pub(crate) struct TextureSlotId {
    idx: u32,
    version: NonZeroU32,
}

impl fmt::Debug for TextureSlotId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}v{}", self.idx, self.version.get())
    }
}

impl TextureSlotId {
    fn new(idx: u32, version: u32) -> Self {
        debug_assert!(version > 0);

        Self {
            idx,
            version: unsafe { NonZeroU32::new_unchecked(version | 1) },
        }
    }
}

// Storage inside a slot or metadata for the freelist when vacant.
union SlotUnion {
    value: TextureId,
    next_free: u32,
}

// A slot, which represents storage for a value and a current version.
// Can be occupied or vacant.
struct Slot {
    u: SlotUnion,
    version: u32, // Even = vacant, odd = occupied.
}

/// Slot map, storage with stable unique keys.
pub(crate) struct TextureIdSlotMap {
    slots: Vec<Slot>,
    free_head: u32,
    num_elems: u32,
}

impl TextureIdSlotMap {
    /// Constructs a new, empty [`TextureIdSlotMap`].
    pub fn new() -> Self {
        let slots = vec![Slot {
            u: SlotUnion { next_free: 0 },
            version: 0,
        }];

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
        self.slots
            .get(key.idx as usize)
            .map_or(false, |slot| slot.version == key.version.get())
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
            let occupied_version = slot.version | 1;
            let kd = TextureSlotId::new(self.free_head, occupied_version);

            // Update.
            unsafe {
                self.free_head = slot.u.next_free;
                slot.u.value = value;
                slot.version = occupied_version;
            }
            self.num_elems = new_num_elems;
            return kd;
        }

        let version = 1;
        let kd = TextureSlotId::new(self.slots.len() as u32, version);

        // Create new slot before adjusting freelist in case f or the allocation panics or errors.
        self.slots.push(Slot {
            u: SlotUnion { value },
            version,
        });

        self.free_head = kd.idx + 1;
        self.num_elems = new_num_elems;
        kd
    }

    /// Removes a key from the slot map if it is present.
    pub fn remove(&mut self, key: TextureSlotId) {
        if self.contains_key(key) {
            let idx = key.idx as usize;

            // This is safe because we know that the slot is occupied.
            let slot = unsafe { self.slots.get_unchecked_mut(idx) };

            // Maintain freelist.
            slot.u.next_free = self.free_head;
            self.free_head = idx as u32;
            self.num_elems -= 1;
            slot.version = slot.version.wrapping_add(1);
        }
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: TextureSlotId) -> Option<TextureId> {
        self.slots
            .get(key.idx as usize)
            .filter(|slot| slot.version == key.version.get())
            .map(|slot| unsafe { slot.u.value })
    }
}

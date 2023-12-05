#[derive(Clone, Copy, Debug)]
pub struct GenerationalId {
    id: usize,
    generation: usize,
}

struct GenerationalCell<T> {
    generation: usize,
    state: T,
}

pub(crate) struct GenerationalStorage<T> {
    vec: Vec<Option<GenerationalCell<T>>>,
    free_indices: Vec<(usize, usize)>,
}

impl<T> GenerationalStorage<T> {
    pub fn new() -> GenerationalStorage<T> {
        GenerationalStorage {
            vec: Vec::with_capacity(1000),
            free_indices: Vec::with_capacity(100),
        }
    }

    pub fn push(&mut self, data: T) -> GenerationalId {
        let generation;

        let id = if let Some((free_id, old_generation)) = self.free_indices.pop() {
            assert!(self.vec[free_id].is_none());

            generation = old_generation + 1;
            self.vec[free_id] = Some(GenerationalCell {
                state: data,
                generation,
            });
            free_id
        } else {
            generation = 0;
            self.vec.push(Some(GenerationalCell {
                state: data,
                generation,
            }));
            self.vec.len() - 1
        };

        GenerationalId { id, generation }
    }

    pub fn get(&self, id: GenerationalId) -> Option<&T> {
        if id.id > self.vec.len() {
            return None;
        }

        if self.vec[id.id].is_none() {
            return None;
        }
        let cell = self.vec[id.id].as_ref().unwrap();
        if cell.generation != id.generation {
            return None;
        }

        Some(&cell.state)
    }

    pub fn get_mut(&mut self, id: GenerationalId) -> Option<&mut T> {
        if id.id > self.vec.len() {
            return None;
        }

        if self.vec[id.id].is_none() {
            return None;
        }
        let cell = self.vec[id.id].as_mut().unwrap();
        if cell.generation != id.generation {
            return None;
        }

        Some(&mut cell.state)
    }

    /// Retains only the elements specified by the predicate, passing a mutable reference to it.

    /// In other words, remove all elements e such that f(&mut e) returns false. This method operates in place, visiting each element exactly once in the original order, and preserves the order of the retained elements.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        for (id, cell) in self.vec.iter_mut().enumerate() {
            if cell.is_none() {
                continue;
            }

            let c = cell.as_mut().unwrap();
            let pred = f(&mut c.state);
            let old_generation = c.generation;

            if !pred {
                self.free_indices.push((id, old_generation));
                *cell = None;
            }
        }
    }

    pub fn count(&self) -> usize {
        self.vec.iter().filter(|c| c.is_some()).count()
    }

    pub fn clear(&mut self) {
        self.vec.clear();
        self.free_indices.clear();
    }

    pub fn free(&mut self, id: GenerationalId) {
        // an attempt to free a cell by an outdated ID
        // this is a legit request, no need to panic or anything, just
        // dont ruin the data that lives there now
        if let Some(cell) = &self.vec[id.id] {
            if cell.generation != id.generation {
                return;
            }
        }

        self.free_indices.push((id.id, id.generation));
        self.vec[id.id] = None;
    }

    pub(crate) fn allocated_memory(&self) -> usize {
        self.vec.capacity() * std::mem::size_of::<GenerationalCell<T>>()
    }
}

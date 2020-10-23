use std::{cell::UnsafeCell, ops::Drop};

#[derive(Clone, Copy, Debug)]
pub struct MagicHandle(Option<usize>);

impl MagicHandle {
    pub fn null() -> MagicHandle {
        MagicHandle(None)
    }
}
pub struct MagicRef<T: 'static> {
    pub data: &'static mut T,
    pub handle: MagicHandle,
    used: *mut bool,
}

impl<T: 'static> Drop for MagicRef<T> {
    fn drop(&mut self) {
        assert_eq!(unsafe { *self.used }, true);
        unsafe {
            *self.used = false;
        }
    }
}

struct MagicCell<T> {
    id: usize,
    data: UnsafeCell<T>,
    used: UnsafeCell<bool>,
}

impl<T> MagicCell<T> {
    fn new(id: usize, data: T) -> Self {
        MagicCell {
            id,
            data: UnsafeCell::new(data),
            used: UnsafeCell::new(false),
        }
    }
}

pub struct MagicVec<T> {
    data: Vec<MagicCell<T>>,
    capacity: usize,
}

impl<T: 'static> MagicVec<T> {
    pub fn new(capacity: usize) -> Self {
        MagicVec {
            data: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn clear(&mut self) {
        for cell in &self.data {
            assert!(unsafe { *cell.used.get() == false });
        }

        self.data.clear()
    }

    pub fn get(&self, handle: MagicHandle) -> Option<MagicRef<T>> {
        if handle.0.is_none() {
            return None;
        }
        let handle = handle.0.unwrap();
        let cell = &self.data[handle];

        if unsafe { *cell.used.get() } {
            return None;
        }

        unsafe { *cell.used.get() = true };

        Some(MagicRef {
            data: unsafe { &mut *cell.data.get() },
            handle: MagicHandle(Some(cell.id)),
            used: cell.used.get(),
        })

    }

    pub fn iter(&self) -> MagicVecIterator<'static, T> {
        let iter = unsafe { std::mem::transmute(self.data.iter()) };
        MagicVecIterator { iter }
    }

    pub fn push(&mut self, data: T) {
        assert!(self.data.len() + 1 < self.capacity);

        self.data.push(MagicCell::new(self.data.len(), data));
    }
}

pub struct MagicVecIterator<'a, T: 'static> {
    iter: std::slice::Iter<'a, MagicCell<T>>,
}

impl<'a, T: 'static> Iterator for MagicVecIterator<'a, T> {
    type Item = MagicRef<T>;

    fn next(&mut self) -> Option<MagicRef<T>> {
        let cell = self.iter.next()?;

        if unsafe { *cell.used.get() } {
            return self.next();
        }

        unsafe { *cell.used.get() = true };

        Some(MagicRef {
            data: unsafe { &mut *cell.data.get() },
            handle: MagicHandle(Some(cell.id)),
            used: cell.used.get(),
        })
    }
}

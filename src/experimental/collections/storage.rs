//! Global read-only storage
//!
//! ```
//! use macroquad::collections::storage;
//!
//! struct WorldBoundries(i32);
//!
//! fn draw_player() {
//!   let boundries: i32 = storage::get::<WorldBoundries>().unwrap().0;
//!   assert_eq!(boundries, 23);
//! }
//!
//! storage::store(WorldBoundries(23));
//! draw_player();
//! ```

use std::any::{Any, TypeId};

use std::collections::HashMap;
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

static mut STORAGE: Option<HashMap<TypeId, Box<dyn Any>>> = None;

/// Store data in global storage.
/// Will silently overwrite an old value if any.
pub fn store<T: Any>(data: T) {
    unsafe {
        if STORAGE.is_none() {
            STORAGE = Some(HashMap::new());
        }

        STORAGE
            .as_mut()
            .unwrap()
            .insert(TypeId::of::<T>(), Box::new(Rc::new(RefCell::new(data))))
    };
}

/// Get reference to data from global storage.
/// Will return None if there is no data available with this type.
pub fn get<T: Any>() -> Option<impl Deref<Target = T>> {
    unsafe {
        if STORAGE.is_none() {
            STORAGE = Some(HashMap::new());
        }

        STORAGE.as_mut().unwrap().get(&TypeId::of::<T>()).as_ref()
    }
    .and_then(|data| {
        data.downcast_ref::<Rc<RefCell<T>>>()
            .map(|data| data.borrow())
    })
}

/// Get mutable reference to data from global storage.
/// Will return None if there is no data available with this type.
pub fn get_mut<T: Any>() -> Option<impl DerefMut<Target = T>> {
    unsafe {
        if STORAGE.is_none() {
            STORAGE = Some(HashMap::new());
        }

        STORAGE.as_mut().unwrap().get(&TypeId::of::<T>()).as_ref()
    }
    .and_then(|data| {
        data.downcast_ref::<Rc<RefCell<T>>>()
            .map(|data| data.borrow_mut())
    })
}

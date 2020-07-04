//! Global read-only storage
//!
//! ```
//! use macroquad::storage;
//!
//! const WORLD_BOUNDRIES: usize = 1;
//!
//! fn draw_player() {
//!   let boundries: i32 = *storage::get(WORLD_BOUNDRIES).unwrap();
//!   assert_eq!(boundries, 23);
//! }
//! 
//! storage::store::<i32>(WORLD_BOUNDRIES, 23);
//! draw_player();
//! ```

use std::any::Any;

use std::{ops::Deref, rc::Rc};

const MAX_ID: usize = 32;
static mut STORAGE: [Option<Box<dyn Any>>; MAX_ID] = [
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
];

/// Store data in global storage.
/// Will panic on incorrect id.
/// And will silently overwrite an old value if any.
pub fn store<T: Any>(id: usize, data: T) {
    assert!(id < MAX_ID);

    unsafe { STORAGE[id] = Some(Box::new(Rc::new(data))) };
}

/// Get data from global storage.
/// Will return None either if id is invalid, requested type do not match stored one or there is no data available with this id.
pub fn get<T: Any>(id: usize) -> Option<impl Deref<Target = T>> {
    unsafe { STORAGE[id].as_ref() }
        .and_then(|data| data.downcast_ref::<Rc<T>>().map(|data| data.clone()))
}

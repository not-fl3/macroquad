use std::cell::UnsafeCell;

/// Cell type that should be preferred over a `static mut` is better to use in a
/// `static`
///
/// Based on [@Nemo157 comment](issue-53639)
/// [issue-53639]: https://github.com/rust-lang/rust/issues/53639#issuecomment-790091647
///
/// # Safety
///
/// Mutable references must never alias (point to the same memory location).
/// Any previous result from calling this method on a specific instance
/// **must** be dropped before calling this function again. This applies
/// even if the mutable refernces lives in the stack frame of another function.
#[repr(transparent)]
pub struct RacyCell<T>(UnsafeCell<T>);

impl<T> RacyCell<T> {
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        RacyCell(UnsafeCell::new(value))
    }

    /// Get a shared reference to the inner type.
    ///
    /// # Safety
    /// See [RacyCell]
    #[inline(always)]
    pub unsafe fn get_ref(&self) -> &T {
        &*self.0.get()
    }

    /// Get a mutable reference to the inner type.
    /// Callers should try to restrict the lifetime as much as possible.
    ///
    /// Callers may want to convert this result to a `*mut T` immediately.
    ///
    /// # Safety
    /// See [RacyCell]
    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    pub unsafe fn get_ref_mut(&self) -> &mut T {
        &mut *self.0.get()
    }

    /// Get a const pointer to the inner type
    ///
    /// # Safety
    /// See [RacyCell]
    #[inline(always)]
    pub unsafe fn get_ptr(&self) -> *const T {
        self.0.get()
    }

    /// Get a mutable pointer to the inner type
    ///
    /// # Safety
    /// See [RacyCell]
    #[inline(always)]
    pub unsafe fn get_ptr_mut(&self) -> *mut T {
        self.0.get()
    }
}

unsafe impl<T> Sync for RacyCell<T> {}

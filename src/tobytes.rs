use crate::math::{Mat4, Vec2, Vec3, Vec4};

/// # Safety
///
/// Implementing this trait declares that the type in question has no padding
/// bytes and can be safely transmuted into `[u8; N]` of the appropriate size.
pub unsafe trait ToBytes {
    fn to_bytes(&self) -> &[u8];
}

macro_rules! impl_tobytes {
    ($t:tt) => {
        unsafe impl ToBytes for $t {
            fn to_bytes(&self) -> &[u8] {
                unsafe {
                    std::slice::from_raw_parts(self as *const _ as *const u8, size_of::<$t>())
                }
            }
        }
    };
}

impl_tobytes!(i32);
impl_tobytes!(f32);
impl_tobytes!(Vec2);
impl_tobytes!(Vec3);
impl_tobytes!(Vec4);
impl_tobytes!(Mat4);

unsafe impl<T: ToBytes, const N: usize> ToBytes for [T; N] {
    fn to_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self as *const _ as *const u8, size_of::<T>() * N) }
    }
}

unsafe impl<T: ToBytes> ToBytes for [T] {
    fn to_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ptr() as *const _ as *const u8,
                size_of::<T>() * self.len(),
            )
        }
    }
}

mod C;
use std::*;
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
pub struct CArray<'a> {
    ptr: *mut u8,
    size: usize,
    auto_free: bool,
    mkr: marker::PhantomData<&'a u8>,
}
pub fn new_carray<'a>(ptr: *mut u8, size: usize, auto_free: bool) -> CArray<'a> {
    CArray {
        ptr,
        size,
        auto_free,
        mkr: marker::PhantomData,
    }
}
impl<'a> CArray<'a> {
    pub fn new(ptr: *mut u8, size: usize, auto_free: bool) -> Self {
        CArray {
            ptr,
            size,
            auto_free,
            mkr: marker::PhantomData,
        }
    }
    pub fn get_slice(&self) -> Result<&[u8]> {
        unsafe {
            if self.ptr.is_null() {
                return Err(format!("ptr is null"))?;
            }
            Ok(slice::from_raw_parts(self.ptr, self.size))
        }
    }
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
}
impl<'a> Drop for CArray<'a> {
    fn drop(&mut self) {
        if self.auto_free {
            unsafe {
                if !self.ptr.is_null() {
                    C::free(*self.ptr as *mut ffi::c_void);
                }
            }
        }
    }
}

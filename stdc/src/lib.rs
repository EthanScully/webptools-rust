mod C;
use std::*;
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
pub struct CArray {
    ptr: *mut u8,
    size: usize,
}
impl CArray {
    pub fn new(ptr: *mut u8, size: usize) -> Self {
        CArray { ptr, size }
    }
    pub fn get_slice(&self) -> Result<&[u8]> {
        unsafe {
            if self.ptr.is_null() {
                return Err(format!("ptr is null"))?;
            }
            Ok(slice::from_raw_parts(self.ptr, self.size))
        }
    }
}
impl Drop for CArray {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                C::free(self.ptr as *mut ffi::c_void);
            }
        }
    }
}

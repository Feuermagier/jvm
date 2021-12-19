use std::alloc::Layout;

#[repr(C)]
pub struct NativeList<T: Copy> {
    list: *mut T,
    length: usize,
}

impl<T: Copy> NativeList<T> {
    pub fn alloc(length: usize) -> Self {
        let layout = Layout::from_size_align(length * 8, 8).unwrap();
        let list = unsafe { std::alloc::alloc(layout) as *mut T };
        Self { list, length }
    }

    pub unsafe fn get(&self, index: usize) -> T {
        *self.list.offset(index as isize)
    }

    /// We don't require mutable access as we will never reallocate
    pub unsafe fn set(&self, index: usize, value: T) {
        *self.list.offset(index as isize) = value;
    }

    pub unsafe fn get_pointer(&self) -> *mut T {
        self.list
    }

    pub fn list_offset(&self) -> usize {
        memoffset::offset_of!(Self, list)
    }
}

impl<T: Copy> Drop for NativeList<T> {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(
                self.list as *mut u8,
                Layout::from_size_align(self.length * 8, 8).unwrap(),
            );
        }
    }
}

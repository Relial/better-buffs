pub struct Mhfdat(*mut u8);

impl Mhfdat {
    pub fn new(ptr: *mut u8) -> Self {
        Self(ptr)
    }

    pub fn guild_food_entry(&self, entry: usize) -> *mut u8 {
        unsafe {
            let ptr = (self.0 as *mut *mut u8).wrapping_byte_add(0xec).read();
            ptr.wrapping_byte_add(entry * 0x22)
        }
    }
}

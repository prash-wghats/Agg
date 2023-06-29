

pub struct BitsetIterator {
    bits: *const u8,
    mask: u8,
}

impl BitsetIterator {
    pub fn new(bits: *const u8, offset: u32) -> BitsetIterator {
        BitsetIterator {
            bits: unsafe { bits.offset((offset >> 3) as isize) },
            mask: 0x80 >> (offset & 7),
        }
    }

    pub fn inc(&mut self) {
        self.mask >>= 1;
        if self.mask == 0 {
            self.bits = unsafe { self.bits.offset(1) };
            self.mask = 0x80;
        }
    }

    pub fn bit(&self) -> u32 {
        (unsafe { *self.bits & self.mask }) as u32
    }
}

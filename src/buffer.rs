// Buffer trait commun pour String et Uint8Array
pub trait ByteBuffer {
    fn push(&mut self, byte: u8);
    fn push_str(&mut self, s: &str);
}

impl ByteBuffer for String {
    fn push(&mut self, byte: u8) {
        self.push(byte as char);
    }
    fn push_str(&mut self, s: &str) {
        self.push_str(s);
    }
}

use js_sys::Uint8Array;

pub struct JsByteBuffer {
    arr: Uint8Array,
    len: u32,
}

impl JsByteBuffer {
    pub fn new(capacity: usize) -> Self {
        JsByteBuffer {
            arr: Uint8Array::new_with_length(capacity as u32),
            len: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.len as usize
    }
    pub fn as_uint8array(&self) -> Uint8Array {
        self.arr.slice(0, self.len)
    }
    fn ensure_capacity(&mut self, additional: usize) {
        let needed = self.len + additional as u32;
        if needed > self.arr.length() {
            let mut new_cap = self.arr.length().max(8);
            while new_cap < needed {
                new_cap *= 2;
            }
            let new_arr = Uint8Array::new_with_length(new_cap);
            new_arr.set(&self.arr.slice(0, self.len), 0);
            self.arr = new_arr;
        }
    }
}

impl ByteBuffer for JsByteBuffer {
    fn push(&mut self, byte: u8) {
        self.ensure_capacity(1);
        self.arr.set_index(self.len, byte);
        self.len += 1;
    }
    fn push_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.ensure_capacity(bytes.len());
        for &b in bytes {
            self.arr.set_index(self.len, b);
            self.len += 1;
        }
    }
}

#[derive(Debug)]
pub struct Buffer {
    cur: usize,
    vec: Vec<u8>,
}

impl From<Vec<u8>> for Buffer {
    fn from(vec: Vec<u8>) -> Self {
        Buffer::from_vec(vec)
    }
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer::from_vec(vec![])
    }

    fn from_vec(v: Vec<u8>) -> Buffer {
        Buffer { cur: 0, vec: v }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.vec
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn next(&mut self) -> u8 {
        let result = self.vec[self.cur];
        self.cur += 1;
        result
    }

    pub fn remaining_size(&self) -> usize {
        self.len() - self.cur
    }

    pub fn read_u32(&mut self) -> u32 {
        if self.remaining_size() < 4 {
            return 0;
        }

        ((self.next() as u32) << 24)
            | ((self.next() as u32) << 16)
            | ((self.next() as u32) << 8)
            | (self.next() as u32)
    }

    pub fn read_u16(&mut self) -> u16 {
        if self.remaining_size() < 2 {
            return 0;
        }

        ((self.next() as u16) << 8) | (self.next() as u16)
    }

    pub fn read_subbuf(&mut self, size: usize) -> Option<Buffer> {
        if size > self.len() {
            return None;
        }

        let mut ret = Vec::with_capacity(size);
        ret.extend_from_slice(&self.vec[self.cur..self.cur + size]);

        Some(Buffer { vec: ret, cur: 0 })
    }

    pub fn push(&mut self, byte: u8) {
        self.vec.push(byte)
    }

    pub fn reset(&mut self) {
        self.cur = 0;
        self.vec.clear();
    }
}

#[derive(Debug)]
pub struct Buffer<'a> {
    data: &'a[u8],
    position: usize
}

impl<'a> Buffer<'a> {
    pub fn new(data: &'a [u8]) -> Buffer<'a> {
        Buffer {
            data,
            position: 0
        }
    }

    pub fn len(&self) -> usize { self.data.len() }

    pub fn seek(&mut self, pos: usize) {
        self.position = pos;
    }

    pub fn skip(&mut self, length: usize) {
        self.position += length;
    }

    pub fn read(&mut self) -> u8 {
        let byte = self.data[self.position];
        self.position += 1;
        return byte;
    }

    pub fn read_multi(&mut self, length: usize) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::with_capacity(length);
        for i in 0..length {
            result.push(self.data[self.position + i]);
        }
        self.position += length;
        return result;
    }

    pub fn debug_get_pos(&self) -> usize { self.position }
}

mod tests {
    #[test]
    fn read() {
        let mut buf = super::Buffer::new(&[1, 2, 3]);
        assert_eq!(buf.read(), 1, "Read a byte");
        assert_eq!(buf.read(), 2, "Read a byte");
        assert_eq!(buf.read(), 3, "Read a byte");
    }

    #[test]
    fn read_multi() {
        let mut buf = super::Buffer::new(&[1, 2, 3, 4, 5]);
        buf.seek(1);
        let range = buf.read_multi(3);
        assert_eq!(range, [2, 3, 4]);
        assert_eq!(buf.read(), 5);
    }

    #[test]
    fn seek() {
        let mut buf = super::Buffer::new(&[1, 2, 3]);
        buf.seek(2);
        assert_eq!(buf.read(), 3, "Read a byte");
    }
}

#[derive(Debug)]
pub struct BytesReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> BytesReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_at_end(&self) -> bool {
        self.pos == self.len()
    }

    pub fn get_pos(&self) -> usize {
        self.pos
    }

    pub fn get_from(&self, start: usize) -> &'a [u8] {
        &self.bytes[start..self.pos]
    }

    pub fn peek(&self) -> u8 {
        self.bytes[self.pos]
    }

    pub fn skip(&mut self) {
        self.read();
    }

    pub fn read(&mut self) -> u8 {
        self.read_n(1)[0]
    }

    pub fn read_n(&mut self, len: usize) -> &'a [u8] {
        let start = self.pos;
        self.pos += len;
        &self.bytes[start..self.pos]
    }

    pub fn read_until(&mut self, byte: u8) -> &'a [u8] {
        let start = self.pos;
        self.pos += self.bytes[start..].iter().position(|x| *x == byte).unwrap();
        &self.bytes[start..self.pos]
    }
}

#[cfg(test)]
mod tests {
    use super::BytesReader;

    #[test]
    fn test_peek() {
        let arr = [1, 2, 3];
        let reader = BytesReader::new(&arr);
        assert_eq!(reader.peek(), 1);
        assert_eq!(reader.peek(), 1);
    }

    #[test]
    fn test_skip() {
        let arr = [1, 2, 3];
        let mut reader = BytesReader::new(&arr);
        reader.skip();
        assert_eq!(reader.peek(), 2);
    }

    #[test]
    fn test_read() {
        let arr = [1, 2, 3];
        let mut reader = BytesReader::new(&arr);
        assert_eq!(reader.read(), 1);
        assert_eq!(reader.read(), 2);
        assert_eq!(reader.read(), 3);
    }

    #[test]
    fn test_read_range() {
        let arr = [1, 2, 3];
        let mut reader = BytesReader::new(&arr);
        assert_eq!(reader.read_n(2), [1, 2]);
        assert_eq!(reader.peek(), 3);
    }

    #[test]
    fn test_read_until() {
        let arr = [1, 2, 3, 4];
        let mut reader = BytesReader::new(&arr);
        reader.skip();
        assert_eq!(reader.read_until(4), [2, 3]);
        assert_eq!(reader.peek(), 4);
    }
}

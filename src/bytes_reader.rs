pub struct BytesReader<'a> {
    bytes: &'a [u8],
}

impl<'a> BytesReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn peek(&self) -> u8 {
        self.bytes[0]
    }

    pub fn skip(&mut self) {
        self.read();
    }

    pub fn read(&mut self) -> u8 {
        self.read_range(1)[0]
    }

    pub fn read_range(&mut self, len: usize) -> &'a [u8] {
        let result = &self.bytes[..len];
        self.bytes = &self.bytes[len..];
        result
    }

    pub fn read_until(&mut self, byte: u8) -> &'a [u8] {
        let pos = self.bytes.iter().position(|x| *x == byte).unwrap();
        let result = &self.bytes[..pos];
        self.bytes = &self.bytes[pos..];
        result
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
        assert_eq!(reader.read_range(2), [1, 2]);
        assert_eq!(reader.peek(), 3);
    }

    #[test]
    fn test_read_until() {
        let arr = [1, 2, 3];
        let mut reader = BytesReader::new(&arr);
        assert_eq!(reader.read_until(3), [1, 2]);
        assert_eq!(reader.peek(), 3);
    }
}

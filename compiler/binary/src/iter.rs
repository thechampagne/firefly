use core::iter;

use crate::{Bitstring, Selection};

/// Represents iteration over the bytes in a selection, which may constitute
/// either a binary or bitstring.
///
/// Iteration may produce a trailing partial byte, of which all unused bits will
/// be zeroed.
pub struct ByteIter<'a> {
    selection: Selection<'a>,
}
impl<'a> Clone for ByteIter<'a> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            selection: self.selection,
        }
    }
}
impl<'a> ByteIter<'a> {
    pub fn new(selection: Selection<'a>) -> Self {
        Self { selection }
    }

    pub fn from_slice(data: &'a [u8]) -> Self {
        Self {
            selection: Selection::all(data),
        }
    }
}
impl<'a> Iterator for ByteIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.selection.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.len()
    }
}
impl<'a> iter::ExactSizeIterator for ByteIter<'a> {
    fn len(&self) -> usize {
        self.selection.byte_size()
    }

    fn is_empty(&self) -> bool {
        self.selection.byte_size() == 0
    }
}
impl<'a> iter::FusedIterator for ByteIter<'a> {}
unsafe impl<'a> iter::TrustedLen for ByteIter<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_iter_test_aligned_binary() {
        let bytes = b"hello";

        let selection = Selection::new(bytes.as_slice(), 0, 0, None, 40).unwrap();
        let mut iter = ByteIter::new(selection);

        assert_eq!(iter.next(), Some(104));
        assert_eq!(iter.next(), Some(101));
        assert_eq!(iter.next(), Some(108));
        assert_eq!(iter.next(), Some(108));
        assert_eq!(iter.next(), Some(111));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn byte_iter_test_aligned_bitstring() {
        let bytes = b"hello";

        let selection = Selection::new(bytes.as_slice(), 0, 0, None, 30).unwrap();
        let mut iter = ByteIter::new(selection);

        assert_eq!(iter.next(), Some(104));
        assert_eq!(iter.next(), Some(101));
        assert_eq!(iter.next(), Some(108));
        assert_eq!(iter.next(), Some(0b01101100));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn byte_iter_test_unaligned() {
        let bytes = b"hello";

        let selection = Selection::new(bytes.as_slice(), 0, 1, None, 39).unwrap();
        let mut iter = ByteIter::new(selection);

        assert_eq!(iter.next(), Some(0b11010000));
        assert_eq!(iter.next(), Some(0b11001010));
        assert_eq!(iter.next(), Some(0b11011000));
        assert_eq!(iter.next(), Some(0b11011000));
        assert_eq!(iter.next(), Some(0b11011110));
        assert_eq!(iter.next(), None);
    }
}

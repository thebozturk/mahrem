//! Unicode code point ile UTF-8 bayt konumu arasında dönüşüm.
//!
//! Veri setleri ve dış API konumları code point cinsinden, bitiş hariç verir
//! (Python `str` indeksleriyle aynı). Rust `str` ise bayt konumuyla dilimlenir.
//! Gerekçe: `docs/decisions/0002-konum-birimi.md`.

use std::ops::Range;

/// Bir metnin code point → bayt konum tablosu.
///
/// ```
/// use mahrem_core::CodePointIndex;
/// let text = "Ayşe'nin";
/// let idx = CodePointIndex::new(text);
/// assert_eq!(idx.len(), 8);
/// assert_eq!(idx.slice(text, 0..4), Some("Ayşe"));
/// assert_eq!(idx.char_of_byte(5), Some(4));
/// ```
#[derive(Debug, Clone)]
pub struct CodePointIndex {
    /// `bytes[i]`: `i`. code point'in bayt konumu; son eleman metin uzunluğu.
    bytes: Vec<usize>,
}

impl CodePointIndex {
    /// Tabloyu kurar.
    #[must_use]
    pub fn new(text: &str) -> Self {
        let mut bytes: Vec<usize> = text.char_indices().map(|(b, _)| b).collect();
        bytes.push(text.len());
        Self { bytes }
    }

    /// Code point sayısı.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len() - 1
    }

    /// Metin boş mu?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Code point konumunu bayt konumuna çevirir. `pos == len()` geçerlidir.
    #[must_use]
    pub fn byte_of_char(&self, pos: usize) -> Option<usize> {
        self.bytes.get(pos).copied()
    }

    /// Bayt konumunu code point konumuna çevirir. Konum bir karakterin
    /// ortasındaysa `None` döner.
    #[must_use]
    pub fn char_of_byte(&self, byte: usize) -> Option<usize> {
        self.bytes.binary_search(&byte).ok()
    }

    /// Code point aralığını bayt aralığına çevirir.
    #[must_use]
    pub fn byte_range(&self, range: Range<usize>) -> Option<Range<usize>> {
        if range.start > range.end {
            return None;
        }
        Some(self.byte_of_char(range.start)?..self.byte_of_char(range.end)?)
    }

    /// Metni code point aralığıyla dilimler. `text` tablonun kurulduğu metin olmalıdır.
    #[must_use]
    pub fn slice<'a>(&self, text: &'a str, range: Range<usize>) -> Option<&'a str> {
        text.get(self.byte_range(range)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii() {
        let idx = CodePointIndex::new("abc");
        assert_eq!(idx.len(), 3);
        assert_eq!(idx.byte_of_char(3), Some(3));
        assert_eq!(idx.byte_of_char(4), None);
    }

    #[test]
    fn multibyte() {
        let text = "İğne 🙂 x";
        let idx = CodePointIndex::new(text);
        assert_eq!(idx.len(), text.chars().count());
        assert_eq!(idx.slice(text, 5..6), Some("🙂"));
        assert_eq!(idx.char_of_byte(1), None);
        #[allow(clippy::reversed_empty_ranges)]
        let reversed = 3..2;
        assert_eq!(idx.byte_range(reversed), None);
    }

    #[test]
    fn empty() {
        let idx = CodePointIndex::new("");
        assert!(idx.is_empty());
        assert_eq!(idx.slice("", 0..0), Some(""));
    }
}

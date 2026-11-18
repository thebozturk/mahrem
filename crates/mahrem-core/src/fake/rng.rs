/// Tohumlu rastgele sayı üreteci (`SplitMix64`).
///
/// Kriptografik değildir; yalnızca tekrarlanabilir sahte veri içindir.
/// Algoritma sabittir: aynı tohum her sürümde ve her platformda aynı diziyi
/// verir.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Tohumdan bir üreteç.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    /// Sonraki 64 bit.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// `0..n` aralığında eşit olasılıklı bir sayı.
    ///
    /// # Panics
    ///
    /// `n` sıfırsa.
    pub fn below(&mut self, n: u64) -> u64 {
        assert!(n > 0, "boş aralık");
        // Modulo yanlılığını önlemek için sığmayan baştaki değerler atılır.
        let zone = u64::MAX - u64::MAX % n;
        loop {
            let x = self.next_u64();
            if x < zone {
                return x % n;
            }
        }
    }

    /// `lo..=hi` aralığında bir sayı.
    ///
    /// # Panics
    ///
    /// `lo > hi` ise.
    pub fn range(&mut self, lo: u64, hi: u64) -> u64 {
        assert!(lo <= hi, "geçersiz aralık");
        if hi - lo == u64::MAX {
            return self.next_u64();
        }
        lo + self.below(hi - lo + 1)
    }

    /// Dilimden bir öğe.
    ///
    /// # Panics
    ///
    /// Dilim boşsa.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        let i = self.below(items.len() as u64);
        &items[usize::try_from(i).expect("dilim boyu usize'a sığar")]
    }

    /// `percent` yüzde olasılıkla `true`.
    pub fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }

    /// `n` rastgele rakam.
    pub(crate) fn digits(&mut self, n: usize) -> String {
        (0..n).map(|_| self.digit()).collect()
    }

    /// `lo..=hi` aralığında bir uzunluk.
    pub(crate) fn size(&mut self, lo: usize, hi: usize) -> usize {
        usize::try_from(self.range(lo as u64, hi as u64)).expect("uzunluk usize'a sığar")
    }

    pub(crate) fn digit(&mut self) -> char {
        self.digit_in(0, 9)
    }

    /// `lo..=hi` aralığında bir rakam karakteri.
    pub(crate) fn digit_in(&mut self, lo: u8, hi: u8) -> char {
        let d = u8::try_from(self.range(u64::from(lo), u64::from(hi))).expect("rakam");
        char::from(b'0' + d)
    }

    /// Büyük bir ASCII harf.
    pub(crate) fn upper_letter(&mut self) -> char {
        char::from(*self.pick(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_is_fixed() {
        // SplitMix64'ün bilinen ilk çıktısı (tohum 0).
        assert_eq!(Rng::new(0).next_u64(), 0xe220_a839_7b1d_cdaf);
    }

    #[test]
    fn below_covers_the_range() {
        let mut rng = Rng::new(1);
        let mut seen = [0u32; 7];
        for _ in 0..7000 {
            seen[usize::try_from(rng.below(7)).unwrap()] += 1;
        }
        assert!(seen.iter().all(|&n| (850..1150).contains(&n)), "{seen:?}");
        assert_eq!(rng.range(5, 5), 5);
        assert!((3..=9).contains(&rng.range(3, 9)));
    }
}

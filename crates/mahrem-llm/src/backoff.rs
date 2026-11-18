//! Üstel bekleme ve jitter.
//!
//! Jitter bir tohumdan türetilir: rastgele sayı crate'i gerekmez ve testler
//! aynı beklemeleri görür. Aynı istemcinin istekleri farklı sayaç değerleri
//! aldığı için beklemeleri yine dağılır.

use std::time::Duration;

/// `attempt`. yeniden denemeden önceki bekleme: `base·2^attempt` (en çok
/// `max`), bunun yarısı sabit, yarısı jitter.
pub(crate) fn delay(base: Duration, max: Duration, attempt: u32, seed: u64) -> Duration {
    let exp = base.saturating_mul(2u32.saturating_pow(attempt)).min(max);
    let half = exp / 2;
    let span = u64::try_from(half.as_micros()).unwrap_or(u64::MAX);
    let jitter = if span == 0 {
        0
    } else {
        splitmix64(seed) % (span + 1)
    };
    half + Duration::from_micros(jitter)
}

/// `SplitMix64` karıştırıcısı.
pub(crate) fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delay_grows_and_is_capped() {
        let base = Duration::from_millis(100);
        let max = Duration::from_secs(1);
        for attempt in 0..10 {
            for seed in 0..50 {
                let d = delay(base, max, attempt, seed);
                let exp = (base * 2u32.pow(attempt)).min(max);
                assert!(d >= exp / 2 && d <= exp, "{attempt} {seed} {d:?}");
            }
        }
        assert_eq!(delay(base, max, 3, 9), delay(base, max, 3, 9));
        assert_ne!(delay(base, max, 3, 9), delay(base, max, 3, 10));
        assert_eq!(delay(Duration::ZERO, max, 3, 9), Duration::ZERO);
        assert!(delay(base, max, u32::MAX, 1) <= max);
    }
}

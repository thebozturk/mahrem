const ONES: [&str; 10] = [
    "", "bir", "iki", "üç", "dört", "beş", "altı", "yedi", "sekiz", "dokuz",
];
const TENS: [&str; 10] = [
    "", "on", "yirmi", "otuz", "kırk", "elli", "altmış", "yetmiş", "seksen", "doksan",
];

/// Sayıyı Türkçe yazıyla yazar: `532` → `beş yüz otuz iki`.
///
/// "bir yüz" ve "bir bin" denmez: `100` → `yüz`, `1000` → `bin`.
///
/// ```
/// use mahrem_core::fake::number_words;
/// assert_eq!(number_words(0), "sıfır");
/// assert_eq!(number_words(1_001), "bin bir");
/// assert_eq!(number_words(2_350_000), "iki milyon üç yüz elli bin");
/// ```
#[must_use]
pub fn number_words(n: u64) -> String {
    if n == 0 {
        return "sıfır".to_owned();
    }
    let mut parts: Vec<String> = Vec::new();
    let groups = [
        (1_000_000_000_000_000_000, "kentilyon"),
        (1_000_000_000_000_000, "katrilyon"),
        (1_000_000_000_000, "trilyon"),
        (1_000_000_000, "milyar"),
        (1_000_000, "milyon"),
        (1_000, "bin"),
    ];
    let mut rest = n;
    for (size, name) in groups {
        let count = rest / size;
        rest %= size;
        if count == 0 {
            continue;
        }
        if !(count == 1 && name == "bin") {
            parts.push(below_thousand(count));
        }
        parts.push(name.to_owned());
    }
    if rest > 0 {
        parts.push(below_thousand(rest));
    }
    parts.join(" ")
}

fn below_thousand(n: u64) -> String {
    let idx = |x: u64| usize::try_from(x).expect("rakam");
    let mut parts = Vec::new();
    let (h, t, o) = (n / 100, (n / 10) % 10, n % 10);
    if h > 1 {
        parts.push(ONES[idx(h)]);
    }
    if h > 0 {
        parts.push("yüz");
    }
    if t > 0 {
        parts.push(TENS[idx(t)]);
    }
    if o > 0 {
        parts.push(ONES[idx(o)]);
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_numbers() {
        for (n, w) in [
            (1, "bir"),
            (10, "on"),
            (11, "on bir"),
            (35, "otuz beş"),
            (100, "yüz"),
            (101, "yüz bir"),
            (200, "iki yüz"),
            (532, "beş yüz otuz iki"),
            (1_000, "bin"),
            (2_000, "iki bin"),
            (11_000, "on bir bin"),
            (100_000, "yüz bin"),
            (101_000, "yüz bir bin"),
            (1_000_000, "bir milyon"),
            (1_000_000_000, "bir milyar"),
            (
                u64::MAX,
                "on sekiz kentilyon dört yüz kırk altı katrilyon yedi yüz kırk dört trilyon yetmiş üç milyar yedi yüz dokuz milyon beş yüz elli bir bin altı yüz on beş",
            ),
        ] {
            assert_eq!(number_words(n), w, "{n}");
        }
    }
}

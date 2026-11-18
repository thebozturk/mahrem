//! LLM çıktısındaki yer tutucuların ayrıştırılması.

use unicode_normalization::UnicodeNormalization as _;

/// Ayrıştırılmış metin parçası.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    /// Düz metin.
    Text(String),
    /// Yer tutucu: planlanmış anahtar (`KİŞİ_1`).
    Slot(String),
}

/// Karşılaştırma için anahtar: ASCII'ye indirgenmiş, büyük harf, boşluk yerine `_`.
pub(crate) fn fold(s: &str) -> String {
    s.trim()
        .chars()
        .map(|c| match c {
            'İ' | 'ı' | 'i' => 'I',
            'Ş' | 'ş' => 'S',
            'Ğ' | 'ğ' => 'G',
            'Ü' | 'ü' => 'U',
            'Ö' | 'ö' => 'O',
            'Ç' | 'ç' => 'C',
            ' ' | '-' => '_',
            c => c.to_ascii_uppercase(),
        })
        .collect()
}

/// Model çıktısındaki sarmalayıcıları temizler: düşünme bloğu, kod bloğu,
/// metni saran tırnak. Metni NFC'ye getirir.
#[must_use]
pub fn clean(raw: &str) -> String {
    let mut s = raw.to_owned();
    while let Some(start) = s.find("<think>") {
        match s[start..].find("</think>") {
            Some(end) => s.replace_range(start..start + end + "</think>".len(), ""),
            None => s.truncate(start),
        }
    }
    let mut s = s.trim();
    if let Some(inner) = s.strip_prefix("```") {
        let inner = inner.split_once('\n').map_or("", |(_, rest)| rest);
        s = inner.strip_suffix("```").unwrap_or(inner).trim();
    }
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') && s.matches('"').count() == 2 {
        s = s[1..s.len() - 1].trim();
    }
    s.replace("\r\n", "\n").nfc().collect()
}

/// Metni parçalara ayırır. Ret gerekçesi kısa ve sabittir.
///
/// # Errors
///
/// Bozuk süslü parantez, bilinmeyen ya da eksik yer tutucu.
pub fn parse(text: &str, keys: &[String]) -> Result<Vec<Piece>, &'static str> {
    let folded: Vec<(String, &String)> = keys.iter().map(|k| (fold(k), k)).collect();
    let mut pieces = Vec::new();
    let mut seen = vec![false; keys.len()];
    let mut rest = text;
    while let Some(open) = rest.find(['{', '}']) {
        if rest[open..].starts_with('}') {
            return Err("bozuk-yer-tutucu");
        }
        let after = &rest[open + 1..];
        let close = after.find('}').ok_or("bozuk-yer-tutucu")?;
        let inner = &after[..close];
        if inner.contains('{') || inner.chars().count() > 40 {
            return Err("bozuk-yer-tutucu");
        }
        let f = fold(inner);
        let i = folded
            .iter()
            .position(|(k, _)| *k == f)
            .ok_or("bilinmeyen-yer-tutucu")?;
        seen[i] = true;
        if open > 0 {
            pieces.push(Piece::Text(rest[..open].to_owned()));
        }
        pieces.push(Piece::Slot(folded[i].1.clone()));
        rest = &after[close + 1..];
    }
    if !rest.is_empty() {
        pieces.push(Piece::Text(rest.to_owned()));
    }
    if seen.iter().any(|s| !s) {
        return Err("eksik-yer-tutucu");
    }
    Ok(pieces)
}

/// Bir yer tutucudan hemen sonra gelen ek: kesme işaretinden sonraki harfler
/// ya da doğrudan bitişik harfler. Küçük harf.
#[must_use]
pub fn suffix_after(next: Option<&Piece>) -> String {
    let Some(Piece::Text(t)) = next else {
        return String::new();
    };
    let t = t.strip_prefix(['\'', '’']).unwrap_or(t);
    mahrem_core::to_lowercase_tr(
        &t.chars()
            .take_while(|c| c.is_alphabetic())
            .collect::<String>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(ks: &[&str]) -> Vec<String> {
        ks.iter().map(|&k| k.to_owned()).collect()
    }

    #[test]
    fn parses_and_matches_loosely() {
        let k = keys(&["KİŞİ_1", "TELEFON_1"]);
        let p = parse("Ben {KISI_1}, numaram {telefon 1}'dir.", &k).unwrap();
        assert_eq!(
            p,
            vec![
                Piece::Text("Ben ".into()),
                Piece::Slot("KİŞİ_1".into()),
                Piece::Text(", numaram ".into()),
                Piece::Slot("TELEFON_1".into()),
                Piece::Text("'dir.".into()),
            ]
        );
        assert_eq!(suffix_after(p.get(4)), "dir");
        assert_eq!(suffix_after(Some(&Piece::Text("’IN evi".into()))), "ın");
        assert_eq!(suffix_after(Some(&Piece::Text(" geldi".into()))), "");
    }

    #[test]
    fn rejects_bad_templates() {
        let k = keys(&["KİŞİ_1"]);
        assert_eq!(parse("Merhaba", &k), Err("eksik-yer-tutucu"));
        assert_eq!(parse("{KİŞİ_1} {AD}", &k), Err("bilinmeyen-yer-tutucu"));
        assert_eq!(parse("{KİŞİ_1", &k), Err("bozuk-yer-tutucu"));
        assert_eq!(parse("{KİŞİ_1} }", &k), Err("bozuk-yer-tutucu"));
        assert!(parse("Kimse yok.", &[]).is_ok());
    }

    #[test]
    fn cleans_wrappers() {
        assert_eq!(clean("<think>plan</think>\n```\nMerhaba\n```"), "Merhaba");
        assert_eq!(clean("\"Merhaba {KİŞİ_1}\""), "Merhaba {KİŞİ_1}");
        assert_eq!(clean("a\r\nb"), "a\nb");
    }
}

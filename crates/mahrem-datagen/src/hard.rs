//! Zor durum dönüşümleri. Parçalar üzerinde çalışır; konumlar kayıt
//! kurulurken yeniden hesaplanır.

use mahrem_core::Label;
use mahrem_core::fake::{Rng, is_ambiguous_name, number_words};
use mahrem_core::{to_lowercase_tr, to_uppercase_tr};
use mahrem_dataset::HardCase;

use crate::doc::Seg;

const NUMERIC: &[Label] = &[
    Label::Phone,
    Label::Tckn,
    Label::Iban,
    Label::CardNumber,
    Label::Vkn,
];

fn strip_diacritics(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'ç' => 'c',
            'Ç' => 'C',
            'ğ' => 'g',
            'Ğ' => 'G',
            'ı' | 'î' => 'i',
            'İ' => 'I',
            'ö' => 'o',
            'Ö' => 'O',
            'ş' => 's',
            'Ş' => 'S',
            'ü' | 'û' => 'u',
            'Ü' => 'U',
            'â' => 'a',
            c => c,
        })
        .collect()
}

fn pick_span(segs: &[Seg], labels: &[Label], rng: &mut Rng) -> Option<usize> {
    let idx: Vec<usize> = segs
        .iter()
        .enumerate()
        .filter(|(_, s)| s.span.is_some_and(|(l, _)| labels.contains(&l)))
        .map(|(i, _)| i)
        .collect();
    if idx.is_empty() {
        None
    } else {
        Some(*rng.pick(&idx))
    }
}

fn typo(word: &str, rng: &mut Rng) -> Option<String> {
    let chars: Vec<char> = word.chars().collect();
    if chars.len() < 4 {
        return None;
    }
    // İç harflerden birini düşür ya da yanındakiyle yer değiştir; ilk ve son harf kalır.
    let i = usize::try_from(rng.range(1, chars.len() as u64 - 3)).ok()?;
    let mut out = chars.clone();
    if rng.chance(50) && out[i] != out[i + 1] {
        out.swap(i, i + 1);
    } else {
        out.remove(i);
    }
    Some(out.into_iter().collect())
}

fn spaced(value: &str, rng: &mut Rng) -> String {
    let chars: Vec<char> = value
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();
    match rng.below(3) {
        0 => chars
            .iter()
            .map(char::to_string)
            .collect::<Vec<_>>()
            .join(" "),
        1 => chars
            .chunks(3)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(" - "),
        _ => chars
            .chunks(2)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("."),
    }
}

fn split_lines(value: &str) -> Option<String> {
    let chars: Vec<char> = value.chars().collect();
    let mid = chars.len() / 2;
    // Ortaya en yakın boşluğu satır sonuna çevir; boşluk yoksa araya ekle.
    let space = (0..chars.len())
        .filter(|&i| chars[i] == ' ')
        .min_by_key(|&i| i.abs_diff(mid));
    let mut out = chars.clone();
    match space {
        Some(i) => out[i] = '\n',
        None if chars.len() >= 4 => out.insert(mid, '\n'),
        None => return None,
    }
    Some(out.into_iter().collect())
}

fn spelled_phone(value: &str) -> String {
    let groups: Vec<String> = if value.contains(' ') {
        value
            .split(' ')
            .map(|g| g.chars().filter(char::is_ascii_digit).collect())
            .filter(|g: &String| !g.is_empty())
            .collect()
    } else {
        let d: String = value.chars().filter(char::is_ascii_digit).collect();
        let cuts = [4, 3, 2, 2];
        let mut out = Vec::new();
        let mut rest = d.as_str();
        for c in cuts {
            if rest.len() <= c {
                break;
            }
            out.push(rest[..c].to_owned());
            rest = &rest[c..];
        }
        out.push(rest.to_owned());
        out
    };
    groups
        .iter()
        .map(|g| {
            let zeros = g.chars().take_while(|&c| c == '0').count();
            let mut words: Vec<String> = vec!["sıfır".to_owned(); zeros];
            let rest = &g[zeros..];
            if !rest.is_empty() {
                words.push(number_words(rest.parse().unwrap_or(0)));
            }
            words.join(" ")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Zor durumu uygular; uygulanamazsa ret gerekçesi döner.
///
/// # Errors
///
/// Belgede dönüşümün gerektirdiği etiketli değer yoksa `zor-durum-uygulanamadi`.
pub fn apply(case: HardCase, segs: &mut [Seg], rng: &mut Rng) -> Result<(), &'static str> {
    const FAIL: &str = "zor-durum-uygulanamadi";
    match case {
        HardCase::NoDiacritics => segs
            .iter_mut()
            .for_each(|s| s.text = strip_diacritics(&s.text)),
        HardCase::AllCaps => segs
            .iter_mut()
            .for_each(|s| s.text = to_uppercase_tr(&s.text)),
        HardCase::AllLowercase => segs
            .iter_mut()
            .for_each(|s| s.text = to_lowercase_tr(&s.text)),
        HardCase::Typo => {
            let i = pick_span(
                segs,
                &[Label::Person, Label::Address, Label::Organization],
                rng,
            )
            .ok_or(FAIL)?;
            let words: Vec<&str> = segs[i].text.split(' ').collect();
            let w = usize::try_from(rng.below(words.len() as u64)).map_err(|_| FAIL)?;
            let changed = typo(words[w], rng).ok_or(FAIL)?;
            let mut new = words.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
            new[w] = changed;
            segs[i].text = new.join(" ");
        }
        HardCase::AmbiguousName => {
            let ok = segs.iter().any(|s| {
                s.span.is_some_and(|(l, _)| l == Label::Person)
                    && s.text.split(' ').any(is_ambiguous_name)
            });
            if !ok {
                return Err(FAIL);
            }
        }
        // Doldurma kişi adı taşıyan adres ya da kurum seçti; burada bir şey yapılmaz.
        HardCase::NameInPlace => {
            pick_span(segs, &[Label::Address, Label::Organization], rng).ok_or(FAIL)?;
        }
        HardCase::SpelledNumbers => {
            let i = pick_span(segs, &[Label::Phone, Label::Age], rng).ok_or(FAIL)?;
            segs[i].text = match segs[i].span {
                Some((Label::Age, _)) => number_words(segs[i].text.parse().map_err(|_| FAIL)?),
                _ => spelled_phone(&segs[i].text),
            };
        }
        HardCase::SpacedNumbers => {
            let i = pick_span(segs, NUMERIC, rng).ok_or(FAIL)?;
            segs[i].text = spaced(&segs[i].text, rng);
        }
        HardCase::SplitLines => {
            let i = pick_span(segs, NUMERIC, rng).ok_or(FAIL)?;
            segs[i].text = split_lines(&segs[i].text).ok_or(FAIL)?;
        }
        HardCase::ObfuscatedEmail => {
            let i = pick_span(segs, &[Label::Email], rng).ok_or(FAIL)?;
            let at = *rng.pick(&[" at ", " [at] ", " (at) "]);
            segs[i].text = segs[i].text.replace('@', at).replace('.', " nokta ");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segs() -> Vec<Seg> {
        vec![
            Seg::plain("Ben "),
            Seg {
                text: "Ayşe Öztürk".into(),
                span: Some((Label::Person, false)),
            },
            Seg::plain(", tel: "),
            Seg {
                text: "0532 123 45 67".into(),
                span: Some((Label::Phone, false)),
            },
            Seg::plain(", e-posta: "),
            Seg {
                text: "ayse.ozturk@gmail.com".into(),
                span: Some((Label::Email, false)),
            },
        ]
    }

    fn after(case: HardCase) -> Vec<Seg> {
        let mut s = segs();
        apply(case, &mut s, &mut Rng::new(3)).unwrap();
        s
    }

    #[test]
    fn transforms() {
        assert_eq!(after(HardCase::NoDiacritics)[1].text, "Ayse Ozturk");
        assert_eq!(after(HardCase::AllCaps)[1].text, "AYŞE ÖZTÜRK");
        assert_eq!(after(HardCase::AllLowercase)[0].text, "ben ");
        assert_eq!(
            after(HardCase::SpelledNumbers)[3].text,
            "sıfır beş yüz otuz iki yüz yirmi üç kırk beş altmış yedi"
        );
        assert!(after(HardCase::SplitLines)[3].text.contains('\n'));
        assert_eq!(
            after(HardCase::ObfuscatedEmail)[5]
                .text
                .matches("nokta")
                .count(),
            2
        );
        let spaced = after(HardCase::SpacedNumbers)[3].text.clone();
        assert_eq!(spaced.chars().filter(char::is_ascii_digit).count(), 11);
        assert_ne!(after(HardCase::Typo)[1].text, "Ayşe Öztürk");
        let mut s = segs();
        assert!(apply(HardCase::AmbiguousName, &mut s, &mut Rng::new(1)).is_err());
    }

    #[test]
    fn age_is_spelled() {
        let mut s = vec![Seg {
            text: "35".into(),
            span: Some((Label::Age, false)),
        }];
        apply(HardCase::SpelledNumbers, &mut s, &mut Rng::new(0)).unwrap();
        assert_eq!(s[0].text, "otuz beş");
    }
}

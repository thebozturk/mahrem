//! Rakam dizileri: TCKN/YKN, VKN, IBAN, kart, telefon.

use super::context::has_context;
use super::{Candidate, Entity};
use crate::Label;
use crate::validators::{is_valid_card_number, is_valid_iban_tr, is_valid_tckn, is_valid_vkn};

/// Rakam grupları arasında izin verilen ayraçlar.
fn is_separator(c: char) -> bool {
    matches!(
        c,
        ' ' | '\u{a0}' | '\t' | '-' | '\u{2013}' | '.' | '/' | '(' | ')' | '\n' | '\r'
    )
}

/// İki grup arasındaki ayraç dizisi en çok bu kadar karakter olabilir
/// (" - " gibi).
const MAX_SEPARATOR: usize = 4;
/// Bir pencerede en çok bu kadar rakam (IBAN 24 + kart 19'un üstü yok).
const MAX_DIGITS: usize = 24;

/// Bir rakam grubu: `[start, end)` code point aralığı.
#[derive(Debug, Clone, Copy)]
struct Group {
    start: usize,
    end: usize,
}

/// Ayraçlarla birleşmiş rakam grupları.
#[derive(Debug)]
struct Run {
    groups: Vec<Group>,
    /// Önünde bitişik ya da tek boşlukla "TR" varsa onun başlangıcı.
    iban_prefix: Option<usize>,
}

fn is_ascii_digit(c: char) -> bool {
    c.is_ascii_digit()
}

/// Rakam bir kodun parçası mı? Önünde harf ya da rakam, veya harfe bitişik
/// `-`/`_` varsa ("TK000111222", "sk-example-0000").
fn attached(chars: &[char], i: usize) -> bool {
    match i.checked_sub(1).map(|j| chars[j]) {
        Some(p) if p.is_alphanumeric() || p == '_' => true,
        Some('-' | '_') => i >= 2 && chars[i - 2].is_alphanumeric(),
        _ => false,
    }
}

/// Önünde "TR" var mı? Bitişik ("TR33") ya da tek boşlukla ("TR 33").
fn iban_prefix(chars: &[char], i: usize) -> Option<usize> {
    let is_tr = |a: usize| {
        chars[a].eq_ignore_ascii_case(&'T')
            && chars[a + 1].eq_ignore_ascii_case(&'R')
            && (a == 0 || !chars[a - 1].is_alphanumeric())
    };
    if i >= 2 && is_tr(i - 2) {
        return Some(i - 2);
    }
    if i >= 3 && chars[i - 1] == ' ' && is_tr(i - 3) {
        return Some(i - 3);
    }
    None
}

fn digit_run_end(chars: &[char], mut i: usize) -> usize {
    while i < chars.len() && is_ascii_digit(chars[i]) {
        i += 1;
    }
    i
}

fn runs(chars: &[char]) -> Vec<Run> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !is_ascii_digit(chars[i]) {
            i += 1;
            continue;
        }
        let prefix = iban_prefix(chars, i);
        let end = digit_run_end(chars, i);
        if prefix.is_none() && attached(chars, i) {
            i = end;
            continue;
        }
        let mut groups = vec![Group { start: i, end }];
        let mut j = end;
        loop {
            let mut k = j;
            let mut newlines = 0;
            while k < chars.len() && is_separator(chars[k]) && k - j < MAX_SEPARATOR + 1 {
                newlines += usize::from(chars[k] == '\n');
                k += 1;
            }
            let sep = k - j;
            if sep == 0
                || sep > MAX_SEPARATOR
                || newlines > 1
                || k >= chars.len()
                || !is_ascii_digit(chars[k])
            {
                break;
            }
            let e = digit_run_end(chars, k);
            groups.push(Group { start: k, end: e });
            j = e;
        }
        i = j;
        out.push(Run {
            groups,
            iban_prefix: prefix,
        });
    }
    out
}

/// Bir pencere: grupların `[first, last]` aralığı.
struct Window<'a> {
    chars: &'a [char],
    run: &'a Run,
    first: usize,
    last: usize,
    digits: String,
}

impl Window<'_> {
    fn start(&self) -> usize {
        self.run.groups[self.first].start
    }

    fn end(&self) -> usize {
        self.run.groups[self.last].end
    }

    /// Penceredeki ayraçlar yalnızca nokta ve gruplar en çok üç haneli mi?
    /// IP adresi, sürüm numarası ya da binlik ayraçlı tutar.
    fn dotted(&self) -> bool {
        let gs = &self.run.groups[self.first..=self.last];
        gs.len() > 1
            && gs.iter().all(|g| g.end - g.start <= 3)
            && gs
                .windows(2)
                .all(|w| self.chars[w[0].end..w[1].start].iter().all(|&c| c == '.'))
    }

    fn entity(&self, label: Label, start: usize, score: f32, source: &'static str) -> Entity {
        Entity {
            label,
            start,
            end: self.end(),
            score,
            source,
        }
    }

    fn context(&self, label: Label) -> bool {
        has_context(self.chars, self.start(), self.end(), label)
    }
}

fn boost(score: f32, context: bool, to: f32) -> f32 {
    if context { to.max(score) } else { score }
}

fn iban(w: &Window<'_>) -> Option<Candidate> {
    let prefix = w.run.iban_prefix.filter(|_| w.first == 0)?;
    if w.digits.len() != 24 {
        return None;
    }
    let candidate = format!("TR{}", w.digits);
    is_valid_iban_tr(&candidate).then(|| Candidate {
        entity: w.entity(Label::Iban, prefix, 0.99, "iban"),
        priority: 0,
    })
}

/// Kart ağının öneki ve geçerli uzunlukları.
fn card_network(d: &str) -> bool {
    let len = d.len();
    let p = |n: usize| d.get(..n).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    match d.as_bytes().first() {
        Some(b'4') => [13, 16, 19].contains(&len),
        Some(b'5') => (51..=55).contains(&p(2)) && len == 16,
        Some(b'2') => (2221..=2720).contains(&p(4)) && len == 16,
        Some(b'3') => {
            [34, 37].contains(&p(2)) && len == 15
                || (3528..=3589).contains(&p(4)) && (16..=19).contains(&len)
        }
        Some(b'6') => (p(4) == 6011 || p(2) == 65 || p(2) == 62) && (16..=19).contains(&len),
        Some(b'9') => p(4) == 9792 && len == 16,
        _ => false,
    }
}

fn card(w: &Window<'_>) -> Option<Candidate> {
    if w.dotted() || !card_network(&w.digits) || !is_valid_card_number(&w.digits) {
        return None;
    }
    let score = boost(0.9, w.context(Label::CardNumber), 0.99);
    Some(Candidate {
        entity: w.entity(Label::CardNumber, w.start(), score, "card"),
        priority: 1,
    })
}

fn tckn(w: &Window<'_>) -> Option<Candidate> {
    if w.dotted() || w.digits.len() != 11 || !is_valid_tckn(&w.digits) {
        return None;
    }
    let (label, source) = if w.digits.starts_with("99") {
        (Label::Ykn, "ykn")
    } else {
        (Label::Tckn, "tckn")
    };
    let score = boost(0.95, w.context(label), 0.99);
    Some(Candidate {
        entity: w.entity(label, w.start(), score, source),
        priority: 2,
    })
}

fn vkn(w: &Window<'_>) -> Option<Candidate> {
    if w.dotted() || w.digits.len() != 10 || !is_valid_vkn(&w.digits) {
        return None;
    }
    // Rastgele 10 haneli sayıların yaklaşık onda biri checksum'dan geçer;
    // bağlam yoksa skor varsayılan eşiğin altında kalır.
    let score = boost(0.4, w.context(Label::Vkn), 0.95);
    Some(Candidate {
        entity: w.entity(Label::Vkn, w.start(), score, "vkn"),
        priority: 3,
    })
}

fn phone(w: &Window<'_>) -> Option<Candidate> {
    if w.dotted() {
        return None;
    }
    let chars = w.chars;
    let start = w.start();
    let plus = start > 0 && chars[start - 1] == '+';
    let d = w.digits.as_str();
    let (national, score) = if plus {
        if let Some(rest) = d.strip_prefix("90").filter(|r| r.len() == 10) {
            (Some(rest), 0.0)
        } else if (8..=15).contains(&d.len()) {
            (None, 0.6)
        } else {
            return None;
        }
    } else if d.len() == 11 && d.starts_with('0') {
        (Some(&d[1..]), 0.0)
    } else if d.len() == 10 && d.starts_with('5') {
        (None, 0.7)
    } else if d.len() == 7 && d.starts_with("444") {
        (None, 0.6)
    } else {
        return None;
    };
    let score = match national {
        Some(n) => match n.as_bytes()[0] {
            b'5' => 0.85,
            b'2' | b'3' | b'4' | b'8' => 0.6,
            _ => return None,
        },
        None => score,
    };
    // Baştaki "+" ve alan kodu parantezi span'in içinde: "+90 (555) …", "(0212) …".
    let area_paren =
        start > 0 && chars[start - 1] == '(' && chars.get(w.run.groups[w.first].end) == Some(&')');
    let span_start = if plus || area_paren { start - 1 } else { start };
    let score = if w.context(Label::Phone) {
        (score + 0.15_f32).min(0.99)
    } else {
        score
    };
    Some(Candidate {
        entity: w.entity(Label::Phone, span_start, score, "phone"),
        priority: 4,
    })
}

/// Rakam dizilerindeki adaylar.
pub(crate) fn scan(chars: &[char]) -> Vec<Candidate> {
    let mut out = Vec::new();
    for run in runs(chars) {
        for first in 0..run.groups.len() {
            let mut digits = String::new();
            for last in first..run.groups.len() {
                let g = run.groups[last];
                digits.extend(&chars[g.start..g.end]);
                if digits.len() > MAX_DIGITS {
                    break;
                }
                let w = Window {
                    chars,
                    run: &run,
                    first,
                    last,
                    digits: digits.clone(),
                };
                out.extend(
                    [iban(&w), card(&w), tckn(&w), vkn(&w), phone(&w)]
                        .into_iter()
                        .flatten(),
                );
            }
        }
    }
    out
}

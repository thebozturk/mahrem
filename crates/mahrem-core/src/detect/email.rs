//! E-posta: `ad@alan.uzantı` ve "at"/"nokta" ile yazılmış biçim.

use super::{Candidate, Entity};
use crate::Label;

fn is_local(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-')
}

fn is_domain(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '-')
}

/// Alan adı geçerli mi: en az bir nokta, harflerden oluşan 2–24 karakterlik uzantı.
fn valid_domain(d: &str) -> bool {
    let Some((head, tld)) = d.rsplit_once('.') else {
        return false;
    };
    !head.is_empty()
        && !head.starts_with(['.', '-'])
        && !head.contains("..")
        && (2..=24).contains(&tld.len())
        && tld.chars().all(|c| c.is_ascii_alphabetic())
}

/// `@` içeren adresler.
fn plain(chars: &[char]) -> Vec<Candidate> {
    let mut out = Vec::new();
    for (at, _) in chars.iter().enumerate().filter(|&(_, &c)| c == '@') {
        let mut s = at;
        while s > 0 && is_local(chars[s - 1]) {
            s -= 1;
        }
        while s < at && matches!(chars[s], '.' | '-' | '+') {
            s += 1;
        }
        let mut e = at + 1;
        while e < chars.len() && is_domain(chars[e]) {
            e += 1;
        }
        while e > at + 1 && matches!(chars[e - 1], '.' | '-') {
            e -= 1;
        }
        let domain: String = chars[at + 1..e].iter().collect();
        if s < at && valid_domain(&domain) {
            out.push(Candidate {
                entity: Entity {
                    label: Label::Email,
                    start: s,
                    end: e,
                    score: 0.99,
                    source: "email",
                },
                priority: 0,
            });
        }
    }
    out
}

/// Gizlenmiş yazımın parçası.
#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Word(String),
    At,
    Dot,
}

/// Boşlukla ayrılmış parçaları ve içlerindeki `[at]`, `(nokta)` gibi
/// işaretleri sırayla çıkarır: (tür, başlangıç, bitiş).
fn tokens(chars: &[char]) -> Vec<(Tok, usize, usize)> {
    const MARKERS: &[(&str, bool)] = &[
        ("[at]", true),
        ("(at)", true),
        ("[nokta]", false),
        ("(nokta)", false),
        ("[dot]", false),
        ("(dot)", false),
    ];
    let lower: Vec<char> = chars.iter().map(char::to_ascii_lowercase).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        let mut word_start = i;
        while i < chars.len() && !chars[i].is_whitespace() {
            let marker = MARKERS.iter().find(|(m, _)| {
                let m: Vec<char> = m.chars().collect();
                lower.get(i..i + m.len()) == Some(&m[..])
            });
            if let Some((m, is_at)) = marker {
                if i > word_start {
                    out.push((
                        Tok::Word(chars[word_start..i].iter().collect()),
                        word_start,
                        i,
                    ));
                }
                let n = m.chars().count();
                out.push((if *is_at { Tok::At } else { Tok::Dot }, i, i + n));
                i += n;
                word_start = i;
            } else {
                i += 1;
            }
        }
        if i > word_start {
            let w: String = chars[word_start..i].iter().collect();
            let tok = match w.to_lowercase().as_str() {
                "at" => Tok::At,
                "nokta" | "dot" => Tok::Dot,
                _ => Tok::Word(w),
            };
            out.push((tok, word_start, i));
        }
    }
    out
}

/// "ayse at example nokta com", "ayse[at]example.com" gibi yazımlar.
fn obfuscated(chars: &[char]) -> Vec<Candidate> {
    let toks = tokens(chars);
    let mut out = Vec::new();
    for i in 0..toks.len() {
        let (Tok::Word(local), ls, _) = &toks[i] else {
            continue;
        };
        if !matches!(toks.get(i + 1), Some((Tok::At, _, _))) || !local.chars().all(is_local) {
            continue;
        }
        // Alan adı: kelime (nokta kelime)+ ya da noktalı tek kelime.
        let mut domain = String::new();
        let mut end = None;
        let mut j = i + 2;
        let mut expect_word = true;
        while let Some((tok, _, e)) = toks.get(j) {
            match (tok, expect_word) {
                (Tok::Word(w), true) => {
                    let w = w.trim_end_matches(|c: char| {
                        matches!(c, ',' | ';' | ':' | '!' | '?' | ')' | '.')
                    });
                    if w.is_empty() || !w.chars().all(is_domain) {
                        break;
                    }
                    let trimmed = toks[j].0 != Tok::Word(w.to_owned());
                    domain.push_str(w);
                    if valid_domain(&domain) {
                        end = Some(*e - (toks_len(&toks[j].0) - w.chars().count()));
                    }
                    if trimmed {
                        break;
                    }
                    expect_word = false;
                }
                (Tok::Dot, false) => {
                    domain.push('.');
                    expect_word = true;
                }
                _ => break,
            }
            j += 1;
        }
        if let Some(end) = end {
            out.push(Candidate {
                entity: Entity {
                    label: Label::Email,
                    start: *ls,
                    end,
                    score: 0.9,
                    source: "email-obfuscated",
                },
                priority: 0,
            });
        }
    }
    out
}

fn toks_len(t: &Tok) -> usize {
    match t {
        Tok::Word(w) => w.chars().count(),
        _ => 0,
    }
}

/// E-posta adayları.
pub(crate) fn scan(chars: &[char]) -> Vec<Candidate> {
    let mut out = plain(chars);
    out.extend(obfuscated(chars));
    out
}

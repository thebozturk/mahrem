//! Mahrem benchmark CLI'ı: bir aracın çıktısını altın sete karşı ölçer.
//!
//! Kullanım: `docs/benchmark.md`. Rapor yalnızca sayı içerir; metin ya da
//! değer yazmaz, bu yüzden kendi verisiyle ölçen biri raporu paylaşabilir.

mod args;
mod report;

use std::collections::HashMap;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use mahrem_dataset::{
    Composition, GoldRecord, GoldSet, PredictedSpan, PredictionRecord, Scope, Severity, Stats,
    digest, evaluate,
};
use sha2::{Digest, Sha256};

use args::Args;

const USAGE: &str = "\
mahrem-bench: Türkçe kişisel veri tespitini altın sete karşı ölçer

Komutlar:
  score --gold <altın.jsonl>... --pred <tahmin.jsonl>
        [--scope all|structural|high-risk] [--min-score 0.5] [--tool <ad>]
        [--json <rapor.json>] [--misses]
      Tahminleri ölçer. Rapor metin ya da değer içermez.

  predict --gold <altın.jsonl>... --out <tahmin.jsonl>
      Mahrem'in yapısal tanıyıcılarıyla tahmin dosyası üretir ve gecikmeyi
      (2.000 karakterde p95) ölçer.

  validate <altın.jsonl>... [--pred <tahmin.jsonl>]
      Altın set ve tahmin dosyalarını doğrular.

  stats <altın.jsonl>... [--composition v0]
      Dağılımı yazar; istenirse kompozisyon hedefleriyle karşılaştırır.
";

const COMPOSITION_V0: &str = include_str!("../../../docs/gold-set/composition-v0.json");

fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let Some((cmd, rest)) = raw.split_first() else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };
    let args = match Args::parse(rest, &["misses"]) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("hata: {e}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let result = match cmd.as_str() {
        "score" => score(&args),
        "predict" => predict(&args),
        "validate" => validate(&args),
        "stats" => stats(&args),
        "-h" | "--help" | "help" => {
            print!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        "-V" | "--version" => {
            println!("mahrem-bench {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        other => Err(format!("bilinmeyen komut: {other}\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("hata: {e}");
            ExitCode::FAILURE
        }
    }
}

fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}

/// Dosyanın yalnızca adı: rapora tam yol (kullanıcı adı içerebilir) yazılmaz.
fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map_or_else(|| path.to_owned(), |n| n.to_string_lossy().into_owned())
}

/// Dosya yolu ve kayıtları.
type GoldFiles = Vec<(String, Vec<GoldRecord>)>;

/// Altın dosyaları doğrular ve okur. Hata varsa ilk bulguları döner.
fn load_gold(paths: &[String]) -> Result<(GoldSet, GoldFiles), String> {
    if paths.is_empty() {
        return Err("en az bir --gold dosyası gerekli".into());
    }
    let mut set = GoldSet::new();
    let mut files = Vec::new();
    for path in paths {
        let content = read(path)?;
        set.add_jsonl(path, &content);
        let records: Vec<GoldRecord> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        files.push((path.clone(), records));
    }
    if set.has_errors() {
        let errors: Vec<String> = set
            .issues()
            .iter()
            .filter(|i| i.severity() == Severity::Error)
            .take(20)
            .map(ToString::to_string)
            .collect();
        return Err(format!("altın set geçersiz:\n{}", errors.join("\n")));
    }
    Ok((set, files))
}

fn score(args: &Args) -> Result<(), String> {
    args.check_known(&["gold", "pred", "scope", "min-score", "tool", "json"])?;
    let (set, files) = load_gold(&args.all("gold"))?;
    let pred_path = args.need("pred")?;
    let pred_content = read(&pred_path)?;
    let issues = set.check_predictions(&pred_path, &pred_content);
    let errors: Vec<String> = issues
        .iter()
        .filter(|i| i.severity() == Severity::Error)
        .take(20)
        .map(ToString::to_string)
        .collect();
    if !errors.is_empty() {
        return Err(format!("tahmin dosyası geçersiz:\n{}", errors.join("\n")));
    }
    let mut predictions: HashMap<String, Vec<PredictedSpan>> = HashMap::new();
    for line in pred_content.lines().filter(|l| !l.trim().is_empty()) {
        let rec: PredictionRecord = serde_json::from_str(line).map_err(|e| e.to_string())?;
        predictions.insert(rec.id, rec.spans);
    }
    let scope: Scope = args.one("scope").map_or(Ok(Scope::All), |s| s.parse())?;
    let min_score: f64 = args.one("min-score").map_or(Ok(0.5), |s| {
        s.parse().map_err(|_| "--min-score sayı olmalı".to_owned())
    })?;
    let gold: Vec<GoldRecord> = files.iter().flat_map(|(_, r)| r.iter().cloned()).collect();
    let (rep, misses) = evaluate(&gold, &predictions, scope, min_score);

    let meta = report::Meta {
        tool: args.one("tool"),
        gold: files
            .iter()
            .map(|(p, r)| (file_name(p), r.len(), digest(r)))
            .collect(),
        predictions: (
            file_name(&pred_path),
            Sha256::digest(pred_content.as_bytes())
                .iter()
                .fold(String::new(), |mut s, b| {
                    use std::fmt::Write as _;
                    let _ = write!(s, "{b:02x}");
                    s
                }),
        ),
    };
    print!("{}", report::text(&rep, &meta));
    if args.flag("misses") {
        print!("{}", report::misses(&misses));
    }
    if let Some(path) = args.one("json") {
        let json =
            serde_json::to_string_pretty(&report::json(&rep, &meta)).map_err(|e| e.to_string())?;
        std::fs::write(&path, json + "\n").map_err(|e| format!("{path}: {e}"))?;
        println!("JSON rapor: {path}");
    }
    Ok(())
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let i = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

fn predict(args: &Args) -> Result<(), String> {
    args.check_known(&["gold", "out"])?;
    let (_, files) = load_gold(&args.all("gold"))?;
    let out_path = args.need("out")?;
    let mut out = String::new();
    let mut n = 0;
    let mut timings = Vec::new();
    for rec in files.iter().flat_map(|(_, r)| r) {
        let spans: Vec<PredictedSpan> = mahrem_core::analyze_with(&rec.text, 0.0)
            .iter()
            .map(|e| PredictedSpan {
                start: e.start,
                end: e.end,
                label: e.label,
                text: None,
                score: Some((f64::from(e.score) * 1000.0).round() / 1000.0),
            })
            .collect();
        let line = serde_json::to_string(&PredictionRecord {
            id: rec.id.clone(),
            spans,
        })
        .map_err(|e| e.to_string())?;
        out.push_str(&line);
        out.push('\n');
        n += 1;
        // PRD: 2.000 karakterlik metinde gecikme. Belge 2.000 karaktere tamamlanır.
        let mut long = String::new();
        while long.chars().count() < 2000 && !rec.text.is_empty() {
            long.push_str(&rec.text);
            long.push('\n');
        }
        let long: String = long.chars().take(2000).collect();
        let start = Instant::now();
        let _ = std::hint::black_box(mahrem_core::analyze_with(std::hint::black_box(&long), 0.0));
        timings.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    std::fs::write(&out_path, out).map_err(|e| format!("{out_path}: {e}"))?;
    timings.sort_by(f64::total_cmp);
    println!("{n} belge için tahmin yazıldı: {out_path}");
    println!(
        "gecikme, 2.000 karakter: p50 {:.2} ms, p95 {:.2} ms, en çok {:.2} ms{}",
        percentile(&timings, 0.5),
        percentile(&timings, 0.95),
        timings.last().copied().unwrap_or(0.0),
        if cfg!(debug_assertions) {
            " (debug derlemesi; ölçüm için --release)"
        } else {
            ""
        }
    );
    Ok(())
}

fn validate(args: &Args) -> Result<(), String> {
    args.check_known(&["pred"])?;
    if args.positional.is_empty() {
        return Err("en az bir altın set dosyası gerekli".into());
    }
    let mut set = GoldSet::new();
    for path in &args.positional {
        set.add_jsonl(path, &read(path)?);
    }
    let mut issues = set.issues().to_vec();
    for path in args.all("pred") {
        issues.extend(set.check_predictions(&path, &read(&path)?));
    }
    for i in &issues {
        println!("{i}");
    }
    let errors = issues
        .iter()
        .filter(|i| i.severity() == Severity::Error)
        .count();
    println!(
        "{} belge, {errors} hata, {} uyarı",
        set.len(),
        issues.len() - errors
    );
    if errors > 0 {
        Err(format!("{errors} hata"))
    } else {
        Ok(())
    }
}

fn stats(args: &Args) -> Result<(), String> {
    args.check_known(&["composition"])?;
    let (_, files) = load_gold(&args.positional)?;
    let gold: Vec<GoldRecord> = files.into_iter().flat_map(|(_, r)| r).collect();
    let s = Stats::from_records(&gold);
    println!("{} belge", s.documents);
    print!("{}", s.summary());
    match args.one("composition").as_deref() {
        None => {}
        Some("v0") => {
            let c = Composition::from_json(COMPOSITION_V0).map_err(|e| e.to_string())?;
            let devs = c.check(&s);
            if devs.is_empty() {
                println!("Kompozisyon v0: hedeflerin hepsi tutuyor.");
            } else {
                println!("Kompozisyon v0, hedeften sapmalar:");
                for d in devs {
                    println!("- {d}");
                }
            }
        }
        Some(other) => {
            return Err(format!(
                "bilinmeyen kompozisyon {other:?}; şimdilik yalnızca v0"
            ));
        }
    }
    Ok(())
}

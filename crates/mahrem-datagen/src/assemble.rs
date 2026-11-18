//! Çalıştırmaları birleştirme, tekilleştirme, bölme ve rapor.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use mahrem_dataset::split::group_of;
use mahrem_dataset::{
    Composition, GoldRecord, GoldSet, Part, Severity, Source, Stats, canonical_jsonl, digest,
    part_of,
};
use serde::Deserialize;

use crate::dedup::dedup;
use crate::error::io;
use crate::{Error, Result};

/// v0 kompozisyon hedefleri.
pub const COMPOSITION_V0: &str = include_str!("../../../docs/gold-set/composition-v0.json");

#[derive(Deserialize)]
struct RunFile {
    run_id: String,
    #[serde(default)]
    provenance: serde_json::Value,
}

#[derive(Deserialize)]
struct ShardLine {
    output: GoldRecord,
}

/// Bir çalıştırma dizininin kimliği, kökeni ve kayıtları. Kayıtlara
/// `origin.run_id` yazılır.
///
/// # Errors
///
/// `run.json` ya da bir shard okunamazsa.
pub fn load_run(dir: &Path) -> Result<(String, serde_json::Value, Vec<GoldRecord>)> {
    let run_path = dir.join("run.json");
    let run: RunFile = serde_json::from_str(&fs::read_to_string(&run_path).map_err(io(&run_path))?)
        .map_err(|e| Error::Invalid(format!("{}: {e}", run_path.display())))?;
    let shards = dir.join("shards");
    let mut paths: Vec<PathBuf> = fs::read_dir(&shards)
        .map_err(io(&shards))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
        .collect();
    paths.sort();
    let mut records = Vec::new();
    for path in paths {
        let text = fs::read_to_string(&path).map_err(io(&path))?;
        for (i, line) in text
            .lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty())
        {
            let mut rec = serde_json::from_str::<ShardLine>(line)
                .map_err(|e| Error::Invalid(format!("{}:{}: {e}", path.display(), i + 1)))?
                .output;
            rec.meta.origin.run_id = Some(run.run_id.clone());
            records.push(rec);
        }
    }
    Ok((run.run_id, run.provenance, records))
}

/// JSONL kayıt dosyasını okur.
///
/// # Errors
///
/// Dosya ya da bir satır okunamazsa.
pub fn load_jsonl(path: &Path) -> Result<Vec<GoldRecord>> {
    let text = fs::read_to_string(path).map_err(io(path))?;
    text.lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
        .map(|(i, l)| {
            serde_json::from_str(l)
                .map_err(|e| Error::Invalid(format!("{}:{}: {e}", path.display(), i + 1)))
        })
        .collect()
}

/// Bir bölümün adı ve hedeften sapmaları.
pub type PartDeviations = (String, Vec<String>);

/// `build` ayarları.
#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// Çalıştırma dizinleri.
    pub runs: Vec<PathBuf>,
    /// Doldurulmuş, elle yazılmış belgeler (JSONL).
    pub handwritten: Vec<PathBuf>,
    /// Tekrar denetiminde karşılaştırılacak setler (zor örnekler gibi).
    pub against: Vec<PathBuf>,
    /// Bölme tohumu.
    pub seed: String,
    /// Açık bölümün payı.
    pub open_share: f64,
    /// Dosya adı öneki (`gold-v0`).
    pub name: String,
    /// Çıktı dizini.
    pub out: PathBuf,
}

/// `build` sonucu.
#[derive(Debug, Clone)]
pub struct BuildReport {
    /// Açık bölümdeki belge sayısı.
    pub open: usize,
    /// Kapalı bölümdeki belge sayısı.
    pub closed: usize,
    /// Tekrar diye atılanlar: (atılan, benzediği).
    pub dropped: Vec<(String, String)>,
    /// Kapalı bölümün özeti.
    pub closed_digest: String,
    /// Hedeften sapmalar, bölüme göre.
    pub deviations: Vec<PartDeviations>,
    /// Raporun yolu.
    pub report: PathBuf,
}

/// Birleştirir, doğrular, tekilleştirir, böler; açık ve kapalı dosyaları ve
/// raporu yazar.
///
/// # Errors
///
/// Girdi okunamazsa ya da doğrulayıcı hata verirse (uyarılar rapora yazılır).
pub fn build(opts: &BuildOptions) -> Result<BuildReport> {
    let mut records = Vec::new();
    let mut runs = Vec::new();
    for dir in &opts.runs {
        let (run_id, provenance, recs) = load_run(dir)?;
        runs.push((run_id, provenance));
        records.extend(recs);
    }
    for path in &opts.handwritten {
        records.extend(load_jsonl(path)?);
    }
    let mut set = GoldSet::new();
    for (i, rec) in records.iter().enumerate() {
        set.add_record("girdi", i + 1, rec);
    }
    let errors: Vec<String> = set
        .issues()
        .iter()
        .filter(|i| i.kind.severity() == Severity::Error)
        .take(20)
        .map(ToString::to_string)
        .collect();
    if !errors.is_empty() {
        return Err(Error::Invalid(format!(
            "doğrulayıcı hataları:\n{}",
            errors.join("\n")
        )));
    }
    let warnings: Vec<String> = set.issues().iter().map(ToString::to_string).collect();

    let mut against = Vec::new();
    for path in &opts.against {
        against.extend(load_jsonl(path)?);
    }
    records.sort_by(|a, b| a.id.as_bytes().cmp(b.id.as_bytes()));
    let dropped = dedup(&mut records, &against);

    let (open, closed): (Vec<GoldRecord>, Vec<GoldRecord>) = records
        .into_iter()
        .partition(|r| part_of(&opts.seed, group_of(r), opts.open_share) == Part::Open);
    fs::create_dir_all(&opts.out).map_err(io(&opts.out))?;
    let open_path = opts.out.join(format!("{}.open.jsonl", opts.name));
    let closed_path = opts.out.join(format!("{}.closed.jsonl", opts.name));
    fs::write(&open_path, canonical_jsonl(&open)).map_err(io(&open_path))?;
    fs::write(&closed_path, canonical_jsonl(&closed)).map_err(io(&closed_path))?;
    let closed_digest = digest(&closed);

    let (deviations, report_path) = write_report(
        opts,
        &runs,
        &open,
        &closed,
        &closed_digest,
        &dropped,
        &warnings,
    )?;
    Ok(BuildReport {
        open: open.len(),
        closed: closed.len(),
        dropped,
        closed_digest,
        deviations,
        report: report_path,
    })
}

fn write_report(
    opts: &BuildOptions,
    runs: &[(String, serde_json::Value)],
    open: &[GoldRecord],
    closed: &[GoldRecord],
    closed_digest: &str,
    dropped: &[(String, String)],
    warnings: &[String],
) -> Result<(Vec<PartDeviations>, PathBuf)> {
    let composition = Composition::from_json(COMPOSITION_V0)
        .map_err(|e| Error::Invalid(format!("composition-v0.json: {e}")))?;
    let mut deviations = Vec::new();
    let mut report = String::new();
    let _ = writeln!(report, "# {}\n", opts.name);
    let _ = writeln!(
        report,
        "- Bölme tohumu: `{}`, açık pay: {}",
        opts.seed, opts.open_share
    );
    let _ = writeln!(report, "- Kapalı bölüm özeti (SHA-256): `{closed_digest}`");
    for (id, prov) in runs {
        let _ = writeln!(
            report,
            "- Çalıştırma `{id}`: model `{}`, sürüm `{}`",
            prov["model"].as_str().unwrap_or("?"),
            prov["model_revision"].as_str().unwrap_or("?")
        );
    }
    let _ = writeln!(report, "- Tekrar diye atılan: {}", dropped.len());
    for (name, part) in [("açık", open), ("kapalı", closed)] {
        let stats = Stats::from_records(part);
        let devs: Vec<String> = composition
            .check(&stats)
            .iter()
            .map(ToString::to_string)
            .collect();
        let _ = writeln!(report, "\n## {name} bölüm: {} belge\n", stats.documents);
        write_stats(&mut report, &stats);
        if devs.is_empty() {
            let _ = writeln!(report, "\nHedeflerin hepsi tutuyor.");
        } else {
            let _ = writeln!(report, "\nHedeften sapmalar:\n");
            for d in &devs {
                let _ = writeln!(report, "- {d}");
            }
        }
        deviations.push((name.to_owned(), devs));
    }
    if !warnings.is_empty() {
        let _ = writeln!(report, "\n## Doğrulayıcı uyarıları\n");
        for w in warnings {
            let _ = writeln!(report, "- {w}");
        }
    }
    if !dropped.is_empty() {
        let _ = writeln!(report, "\n## Tekrarlar\n");
        for (a, b) in dropped {
            let _ = writeln!(report, "- `{a}` → `{b}`");
        }
    }
    let report_path = opts.out.join(format!("{}.report.md", opts.name));
    fs::write(&report_path, report).map_err(io(&report_path))?;
    Ok((deviations, report_path))
}

#[allow(clippy::cast_precision_loss)]
fn write_stats(out: &mut String, s: &Stats) {
    let pct = |n: usize| {
        if s.documents == 0 {
            0.0
        } else {
            n as f64 * 100.0 / s.documents as f64
        }
    };
    let domains: Vec<String> = s
        .domains
        .iter()
        .map(|(d, n)| format!("{d:?} {:.0}%", pct(*n)))
        .collect();
    let _ = writeln!(out, "- Metin türleri: {}", domains.join(", "));
    let buckets: BTreeMap<&str, usize> = [
        ("0", s.documents_with_entities(0, Some(0))),
        ("1", s.documents_with_entities(1, Some(1))),
        ("2-3", s.documents_with_entities(2, Some(3))),
        ("4-5", s.documents_with_entities(4, Some(5))),
        ("6+", s.documents_with_entities(6, None)),
    ]
    .into_iter()
    .collect();
    let b: Vec<String> = buckets
        .iter()
        .map(|(k, n)| format!("{k}: {:.0}%", pct(*n)))
        .collect();
    let _ = writeln!(out, "- Varlık sayısı: {}", b.join(", "));
    let labels: Vec<String> = s
        .label_spans
        .iter()
        .map(|(l, n)| format!("{l} {n}"))
        .collect();
    let _ = writeln!(out, "- Etiketler: {}", labels.join(", "));
    let _ = writeln!(
        out,
        "- Yüksek riskli belge: {}, çocuk span: {}, zor durumlu belge: {:.0}%, elle yazılmış: {:.0}%",
        s.high_risk_documents,
        s.child_spans,
        pct(s.hard_case_documents),
        pct(s.sources.get(&Source::Handwritten).copied().unwrap_or(0))
    );
}

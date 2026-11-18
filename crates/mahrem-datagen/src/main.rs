//! `mahrem-datagen` komut satırı.

use std::path::PathBuf;
use std::process::ExitCode;

use mahrem_datagen::assemble::{BuildOptions, build};
use mahrem_datagen::job::Job;
use mahrem_datagen::stage::GenStage;
use mahrem_datagen::{Error, blind, handwritten};
use mahrem_harness::{Plan, Runner, cancel_on_ctrl_c, init_tracing};

const USAGE: &str = "\
mahrem-datagen: sentetik veri üretimi

Komutlar:
  run <plan.toml> --out <dizin> [--json-logs]
      Planı yerel LLM ile çalıştırır; kesilirse aynı komut kaldığı yerden sürer.

  build --run <dizin>... --seed <tohum> --out <dizin>
        [--handwritten <dosya.jsonl>]... [--against <dosya.jsonl>]...
        [--open-share 0.5] [--name gold-v0]
      Çalıştırmaları birleştirir, tekilleştirir, böler; açık ve kapalı
      bölümü ve raporu yazar.

  fill <girdi.jsonl> --seed <sayı> --out <çıktı.jsonl>
      Yer tutucuyla elle yazılmış metinleri doldurur.

  blind --run <dizin>... --out <dizin> [--per-run 25] [--seed <tohum>]
      Çalıştırmaların metinlerini model adı olmadan karıştırır (model seçimi).
";

struct Args {
    positional: Vec<String>,
    options: Vec<(String, String)>,
    flags: Vec<String>,
}

impl Args {
    fn parse(raw: &[String]) -> Result<Self, String> {
        let mut a = Args {
            positional: Vec::new(),
            options: Vec::new(),
            flags: Vec::new(),
        };
        let mut it = raw.iter();
        while let Some(arg) = it.next() {
            if let Some(name) = arg.strip_prefix("--") {
                if name == "json-logs" {
                    a.flags.push(name.to_owned());
                } else {
                    let value = it.next().ok_or(format!("--{name} değer ister"))?;
                    a.options.push((name.to_owned(), value.clone()));
                }
            } else {
                a.positional.push(arg.clone());
            }
        }
        Ok(a)
    }

    fn all(&self, name: &str) -> Vec<String> {
        self.options
            .iter()
            .filter(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
            .collect()
    }

    fn one(&self, name: &str) -> Option<String> {
        self.all(name).pop()
    }

    fn need(&self, name: &str) -> Result<String, String> {
        self.one(name).ok_or(format!("--{name} gerekli"))
    }

    fn check_known(&self, known: &[&str]) -> Result<(), String> {
        match self
            .options
            .iter()
            .find(|(k, _)| !known.contains(&k.as_str()))
        {
            Some((k, _)) => Err(format!("bilinmeyen seçenek --{k}")),
            None => Ok(()),
        }
    }
}

fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let Some((cmd, rest)) = raw.split_first() else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };
    let args = match Args::parse(rest) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("hata: {e}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let result = match cmd.as_str() {
        "run" => run(&args),
        "build" => cmd_build(&args),
        "fill" => cmd_fill(&args),
        "blind" => cmd_blind(&args),
        "-h" | "--help" | "help" => {
            print!("{USAGE}");
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

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn run(args: &Args) -> Result<(), String> {
    args.check_known(&["out"])?;
    let plan_path = args.positional.first().ok_or("plan dosyası gerekli")?;
    let out = PathBuf::from(args.need("out")?);
    init_tracing(args.flags.iter().any(|f| f == "json-logs"));
    let text = std::fs::read_to_string(plan_path).map_err(|e| format!("{plan_path}: {e}"))?;
    let plan = Plan::from_toml(&text).map_err(err)?;
    let job: Job = plan.job().map_err(err)?;
    let provenance = job.provenance();
    let stage = GenStage::new(job, plan.concurrency).map_err(err)?;
    let rt = tokio::runtime::Runtime::new().map_err(err)?;
    let summary = rt
        .block_on(async {
            Runner::new(plan, &out)
                .provenance(provenance)
                .run(stage, cancel_on_ctrl_c())
                .await
        })
        .map_err(|e| err(Error::from(e)))?;
    println!(
        "çalıştırma {}: kabul {}, ret {}, başarısız {}, kalan {}{}",
        summary.run_id,
        summary.total.done,
        summary.total.rejected,
        summary.total.failed,
        summary.remaining,
        if summary.interrupted {
            " (durduruldu)"
        } else {
            ""
        }
    );
    for (reason, n) in &summary.reject_reasons {
        println!("  ret {reason}: {n}");
    }
    Ok(())
}

fn cmd_build(args: &Args) -> Result<(), String> {
    args.check_known(&[
        "run",
        "handwritten",
        "against",
        "seed",
        "out",
        "open-share",
        "name",
    ])?;
    let opts = BuildOptions {
        runs: args.all("run").into_iter().map(PathBuf::from).collect(),
        handwritten: args
            .all("handwritten")
            .into_iter()
            .map(PathBuf::from)
            .collect(),
        against: args.all("against").into_iter().map(PathBuf::from).collect(),
        seed: args.need("seed")?,
        open_share: args.one("open-share").map_or(Ok(0.5), |s| {
            s.parse().map_err(|_| "--open-share sayı olmalı".to_owned())
        })?,
        name: args.one("name").unwrap_or_else(|| "gold-v0".to_owned()),
        out: PathBuf::from(args.need("out")?),
    };
    if opts.runs.is_empty() && opts.handwritten.is_empty() {
        return Err("en az bir --run ya da --handwritten gerekli".into());
    }
    let r = build(&opts).map_err(err)?;
    println!(
        "açık {}, kapalı {}, tekrar {}; kapalı özet {}",
        r.open,
        r.closed,
        r.dropped.len(),
        r.closed_digest
    );
    for (part, devs) in &r.deviations {
        if !devs.is_empty() {
            println!("{part} bölümde {} sapma", devs.len());
        }
    }
    println!("rapor: {}", r.report.display());
    Ok(())
}

fn cmd_fill(args: &Args) -> Result<(), String> {
    args.check_known(&["seed", "out"])?;
    let input = args.positional.first().ok_or("girdi dosyası gerekli")?;
    let seed: u64 = args
        .need("seed")?
        .parse()
        .map_err(|_| "--seed sayı olmalı")?;
    let out = PathBuf::from(args.need("out")?);
    let (written, failed) =
        handwritten::fill_file(&PathBuf::from(input), seed, &out).map_err(err)?;
    println!("{written} belge dolduruldu: {}", out.display());
    for (id, reason) in &failed {
        println!("  doldurulamadı {id}: {reason}");
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!("{} belge doldurulamadı", failed.len()))
    }
}

fn cmd_blind(args: &Args) -> Result<(), String> {
    args.check_known(&["run", "out", "per-run", "seed"])?;
    let runs: Vec<PathBuf> = args.all("run").into_iter().map(PathBuf::from).collect();
    if runs.is_empty() {
        return Err("en az bir --run gerekli".into());
    }
    let per_run = args.one("per-run").map_or(Ok(25), |s| {
        s.parse().map_err(|_| "--per-run sayı olmalı".to_owned())
    })?;
    let seed = args.one("seed").unwrap_or_else(|| "kor-okuma".to_owned());
    let out = PathBuf::from(args.need("out")?);
    let n = blind::blind(&runs, per_run, &seed, &out).map_err(err)?;
    println!("{n} metin: {}", out.join("metinler.md").display());
    Ok(())
}

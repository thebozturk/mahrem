//! Uçtan uca: sahte LLM sunucusu → üretim → birleştirme → bölme → kör okuma.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use mahrem_datagen::assemble::{BuildOptions, build, load_jsonl, load_run};
use mahrem_datagen::blind::blind;
use mahrem_datagen::job::Job;
use mahrem_datagen::stage::GenStage;
use mahrem_dataset::{GoldSet, Source, digest};
use mahrem_harness::{CancellationToken, Plan, Runner};
use serde_json::{Value, json};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

/// İstekteki yer tutucu listesini okuyup onları kullanan bir metin yazar.
/// Her onuncu istekte bir yer tutucuyu atlar (ret beklenir).
struct FakeLlm;

impl Respond for FakeLlm {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        let body: Value = serde_json::from_slice(&req.body).unwrap();
        let user = body["messages"][1]["content"].as_str().unwrap();
        let seed = body["seed"].as_u64().unwrap();
        let topic = user
            .lines()
            .find_map(|l| l.strip_prefix("Konu: "))
            .unwrap_or("konu");
        let keys: Vec<&str> = user
            .lines()
            .filter_map(|l| l.strip_prefix("- {"))
            .filter_map(|l| l.split_once('}').map(|(k, _)| k))
            .collect();
        let words = [
            "sabah",
            "akşam",
            "dün",
            "bugün",
            "yarın",
            "geçen hafta",
            "az önce",
        ];
        let mut text = format!(
            "{} {topic} hakkında yazıyorum, işlem {} numaralı.",
            words[usize::try_from(seed % 7).unwrap()],
            seed % 100_000
        );
        let skip = seed.is_multiple_of(10) && !keys.is_empty();
        for (i, k) in keys.iter().enumerate() {
            if skip && i == 0 {
                continue;
            }
            let _ = write!(text, " Bilgi {}: {{{k}}}.", i + 1);
        }
        ResponseTemplate::new(200).set_body_json(json!({
            "model": "sahte",
            "choices": [{"index": 0, "message": {"role": "assistant", "content": text}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 100, "completion_tokens": 40}
        }))
    }
}

fn tmp(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "mahrem-datagen-{name}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn hard_cases() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../datasets/hard-cases/hard-cases.jsonl")
}

#[tokio::test(flavor = "multi_thread")]
async fn generate_build_and_blind() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(FakeLlm)
        .mount(&server)
        .await;
    let dir = tmp("e2e");
    let plan = Plan::from_toml(&format!(
        "name = \"deneme\"\nseed = \"tohum\"\nitems = 80\nconcurrency = 4\n\n[job]\nendpoint = \"{}/v1\"\nmodel = \"sahte\"\nmodel_revision = \"abc123\"\n",
        server.uri()
    ))
    .unwrap();
    let job: Job = plan.job().unwrap();
    let provenance = job.provenance();
    let stage = GenStage::new(job, plan.concurrency).unwrap();
    let run_dir = dir.join("run");
    let summary = Runner::new(plan, &run_dir)
        .provenance(provenance)
        .run(stage, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(
        summary.total.done + summary.total.rejected,
        80,
        "{summary:?}"
    );
    assert!(summary.total.done >= 50, "{summary:?}");
    assert!(
        summary
            .reject_reasons
            .iter()
            .any(|(r, _)| r == "eksik-yer-tutucu"),
        "{:?}",
        summary.reject_reasons
    );

    let (_, prov, records) = load_run(&run_dir).unwrap();
    assert_eq!(prov["model_revision"], "abc123");
    assert_eq!(prov["template_hash"].as_str().unwrap().len(), 64);
    assert_eq!(records.len() as u64, summary.total.done);
    for r in &records {
        assert_eq!(r.meta.source, Source::Llm);
        assert!(r.meta.origin.run_id.is_some() && r.meta.origin.group.is_some());
        assert_eq!(r.meta.origin.generator.as_deref(), Some("sahte"));
    }
    assert!(records.iter().any(|r| !r.meta.hard_cases.is_empty()));
    assert!(records.iter().any(|r| r.spans.iter().any(|s| s.child)));

    let out = dir.join("gold");
    let report = build(&BuildOptions {
        runs: vec![run_dir.clone()],
        handwritten: vec![],
        against: vec![hard_cases()],
        seed: "bolme".into(),
        open_share: 0.5,
        name: "deneme".into(),
        out: out.clone(),
    })
    .unwrap();
    assert_eq!(
        report.open + report.closed + report.dropped.len(),
        records.len()
    );
    assert!(report.open > 0 && report.closed > 0);
    let open = load_jsonl(&out.join("deneme.open.jsonl")).unwrap();
    let closed = load_jsonl(&out.join("deneme.closed.jsonl")).unwrap();
    assert_eq!(digest(&closed), report.closed_digest);
    let mut set = GoldSet::new();
    for (i, r) in open.iter().chain(&closed).enumerate() {
        set.add_record("cikti", i + 1, r);
    }
    assert!(set.issues().is_empty(), "{:?}", set.issues());
    // Aynı grup aynı bölümde.
    for o in &open {
        assert!(
            closed
                .iter()
                .all(|c| c.meta.origin.group != o.meta.origin.group)
        );
    }
    let text = std::fs::read_to_string(&report.report).unwrap();
    assert!(
        text.contains("Kapalı bölüm özeti") && text.contains("abc123"),
        "{text}"
    );

    let n = blind(&[run_dir.clone(), run_dir], 5, "kor", &dir.join("kor")).unwrap();
    assert_eq!(n, 10);
    let key: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("kor/anahtar.json")).unwrap())
            .unwrap();
    assert_eq!(key.as_array().unwrap().len(), 10);
    let md = std::fs::read_to_string(dir.join("kor/metinler.md")).unwrap();
    assert!(!md.contains("sahte"), "model adı okuma dosyasında olmamalı");

    std::fs::remove_dir_all(dir).unwrap();
}

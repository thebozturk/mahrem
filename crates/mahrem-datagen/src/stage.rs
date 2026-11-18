//! Harness aşaması ve tek belgelik üretim hattı.

use std::time::Duration;

use mahrem_core::fake::Rng;
use mahrem_dataset::{GoldRecord, Source};
use mahrem_harness::{Item, Stage, StageError};
use mahrem_llm::{ChatRequest, Client, ClientConfig, LlmError};

use crate::check::leak;
use crate::doc::{RecordMeta, record};
use crate::fill::{FillContext, fill};
use crate::hard;
use crate::job::Job;
use crate::prompt::messages;
use crate::spec::{DocSpec, Scenarios, plan};
use crate::template::{clean, parse};

/// LLM çıktısından kayda: ayrıştır, denetle, doldur, dönüştür, doğrula.
///
/// # Errors
///
/// Kısa ret gerekçesi: `eksik-yer-tutucu`, `ham-numara`, `ek-uyumsuz` …
pub fn build_record(
    id: &str,
    spec: &DocSpec,
    raw: &str,
    seed: u64,
    generator: Option<&str>,
) -> Result<GoldRecord, String> {
    let text = clean(raw);
    if text.chars().count() < 20 {
        return Err("cok-kisa".into());
    }
    let keys: Vec<String> = spec.entities.iter().map(|e| e.key.clone()).collect();
    let pieces = parse(&text, &keys)?;
    if let Some(reason) = leak(&pieces) {
        return Err(reason.into());
    }
    let mut rng = Rng::new(seed);
    let fc = FillContext {
        domain: spec.domain,
        hard_case: spec.hard_case,
    };
    let mut segs = fill(&spec.entities, &pieces, fc, &mut rng)?;
    if let Some(case) = spec.hard_case {
        hard::apply(case, &mut segs, &mut rng)?;
    }
    record(
        id,
        &segs,
        RecordMeta {
            domain: spec.domain,
            hard_cases: spec.hard_case.into_iter().collect(),
            source: Source::Llm,
            generator,
            group: Some(spec.group.clone()),
        },
    )
}

/// Yerel LLM ile belge üreten aşama.
pub struct GenStage {
    client: Client,
    job: Job,
    scenarios: Scenarios,
}

impl GenStage {
    /// Aşamayı kurar.
    ///
    /// # Errors
    ///
    /// Uç adresi geçersiz ya da yerel değilse (`allow_remote` kapalıyken).
    pub fn new(job: Job, concurrency: usize) -> crate::Result<Self> {
        let mut config = ClientConfig::local(job.endpoint.clone());
        config.allow_remote = job.allow_remote;
        config.timeout = Duration::from_secs(job.timeout_secs);
        config.concurrency = concurrency;
        Ok(GenStage {
            client: Client::new(config)?,
            job,
            scenarios: Scenarios::embedded(),
        })
    }
}

fn llm_error(e: LlmError) -> StageError {
    match e {
        LlmError::Config(_) | LlmError::RemoteEndpoint(_) | LlmError::Rejected { .. } => {
            StageError::fatal(e)
        }
        _ => StageError::retryable(e),
    }
}

impl Stage for GenStage {
    type Output = GoldRecord;

    async fn run(&self, item: &Item, attempt: u32) -> Result<GoldRecord, StageError> {
        let spec = plan(item.seed, &self.scenarios);
        let attempt_seed = item.seed ^ u64::from(attempt).rotate_left(32);
        let mut req = ChatRequest::new(self.job.model.clone(), messages(&spec, &self.scenarios))
            .temperature(self.job.temperature)
            .max_tokens(self.job.max_tokens)
            .seed(attempt_seed);
        if let Some(p) = self.job.top_p {
            req = req.top_p(p);
        }
        let resp = self.client.chat(&req).await.map_err(llm_error)?;
        if resp.finish_reason.as_deref() == Some("length") {
            return Err(StageError::reject("uzunluk-siniri"));
        }
        build_record(
            &item.id,
            &spec,
            &resp.content,
            attempt_seed.rotate_left(17),
            Some(&self.job.model),
        )
        .map_err(StageError::reject)
    }

    fn counters(&self) -> Vec<(&'static str, u64)> {
        let u = self.client.usage();
        vec![
            ("prompt_tokens", u.prompt_tokens),
            ("completion_tokens", u.completion_tokens),
        ]
    }
}

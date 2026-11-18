//! Çalıştırıcı.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use crate::error::IoContext as _;
use crate::provenance::{new_run_id, record_session};
use crate::shard::{self, Line};
use crate::store::{Status, Store};
use crate::{HarnessError, Item, Plan, Stage, StageError};

/// Duruma göre örnek sayıları.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Tally {
    /// Kabul edilip shard'a yazılan.
    pub done: u64,
    /// Reddedilen.
    pub rejected: u64,
    /// Denemeleri tükenen; sonraki çalıştırmada yeniden denenir.
    pub failed: u64,
}

/// Çalıştırmanın sonucu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSummary {
    /// Çalıştırma kimliği; kaldığı yerden devamda değişmez.
    pub run_id: String,
    /// Bu oturumda işlenenler.
    pub session: Tally,
    /// Çıktı dizinindeki toplam durum.
    pub total: Tally,
    /// Henüz bitmemiş (kabul ya da ret almamış) örnek sayısı.
    pub remaining: u64,
    /// Durdurma isteğiyle mi bitti?
    pub interrupted: bool,
    /// Ret gerekçeleri ve sayıları, en sıktan başlayarak.
    pub reject_reasons: Vec<(String, u64)>,
    /// Açılışta shard'lara göre düzeltilen kayıt sayısı.
    pub repaired: u64,
}

/// Bir planı bir aşamayla çalıştırır.
#[derive(Debug)]
pub struct Runner {
    plan: Plan,
    out_dir: PathBuf,
    provenance: serde_json::Value,
    progress_every: Duration,
}

enum Event {
    Done {
        item: Item,
        attempts: u32,
        line: Vec<u8>,
    },
    Rejected {
        item: Item,
        attempts: u32,
        reason: String,
    },
    Failed {
        item: Item,
        attempts: u32,
        error: String,
    },
}

#[derive(Default)]
struct Counters {
    done: AtomicU64,
    rejected: AtomicU64,
    failed: AtomicU64,
}

impl Counters {
    fn tally(&self) -> Tally {
        Tally {
            done: self.done.load(Ordering::Relaxed),
            rejected: self.rejected.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
        }
    }
}

/// Çalıştırmayı durduran ilk hata.
type Abort = Arc<Mutex<Option<HarnessError>>>;

fn abort(slot: &Abort, stop: &CancellationToken, err: HarnessError) {
    let mut slot = slot
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if slot.is_none() {
        *slot = Some(err);
    }
    stop.cancel();
}

impl Runner {
    /// `out_dir` altında çalışacak bir çalıştırıcı.
    pub fn new(plan: Plan, out_dir: impl Into<PathBuf>) -> Self {
        Runner {
            plan,
            out_dir: out_dir.into(),
            provenance: serde_json::Value::Object(serde_json::Map::new()),
            progress_every: Duration::from_secs(10),
        }
    }

    /// Köken kaydına eklenecek alanlar: model, ağırlık özeti, örnekleme
    /// parametreleri, şablon özeti … Kaldığı yerden devamda aynı olmalı.
    #[must_use]
    pub fn provenance(mut self, extra: serde_json::Value) -> Self {
        self.provenance = extra;
        self
    }

    /// İlerleme logunun sıklığı (varsayılan 10 sn).
    #[must_use]
    pub fn progress_every(mut self, every: Duration) -> Self {
        self.progress_every = every;
        self
    }

    /// Planı çalıştırır. Önceki bir çalıştırma yarım kaldıysa kaldığı yerden
    /// sürer. `cancel` tetiklenince yeni örnek başlatılmaz; süren örnekler
    /// bitirilip yazılır ve `interrupted: true` döner.
    ///
    /// # Errors
    ///
    /// Çıktı dizini başka bir planla ya da başka kökenle başlatılmışsa, shard
    /// bozuksa, dosya ya da veritabanı hatasında ve aşama ölümcül hata
    /// verdiğinde.
    pub async fn run<S: Stage>(
        self,
        stage: S,
        cancel: CancellationToken,
    ) -> Result<RunSummary, HarnessError> {
        let shards_dir = self.out_dir.join("shards");
        fs::create_dir_all(&shards_dir).at(&shards_dir)?;
        let mut store = Store::open(&self.out_dir.join("state.sqlite"))?;
        let run_id = self.check_identity(&store)?;

        let scan = shard::scan(&shards_dir)?;
        if scan.truncated > 0 {
            warn!(bytes = scan.truncated, "son shard'daki yarım satır kesildi");
        }
        let repaired = store.reconcile(&scan.items)?;
        if repaired > 0 {
            info!(repaired, "durum veritabanı shard'lara göre düzeltildi");
        }
        record_session(&self.out_dir, &run_id, &self.plan, &self.provenance)?;
        let finished = store.finished()?;
        info!(
            run_id = %run_id,
            plan = %self.plan.name,
            items = self.plan.items,
            finished = finished.len(),
            "çalıştırma başlıyor"
        );

        let stop = cancel.child_token();
        let failure: Abort = Arc::default();
        let counters = Arc::new(Counters::default());
        let (tx, rx) = mpsc::channel::<Event>(self.plan.concurrency * 2);
        let writer = shard::Writer::new(shards_dir, self.plan.shard_size, scan.last);
        let writer_thread = {
            let counters = Arc::clone(&counters);
            let failure = Arc::clone(&failure);
            let stop = stop.clone();
            std::thread::spawn(move || {
                write_loop(rx, store, writer, &counters).map_err(|e| abort(&failure, &stop, e))
            })
        };

        let stage = Arc::new(stage);
        let progress = self.spawn_progress(Arc::clone(&stage), Arc::clone(&counters));
        self.dispatch(&stage, &finished, &tx, &stop, &failure).await;
        drop(tx);
        progress.abort();

        let store = tokio::task::spawn_blocking(move || writer_thread.join())
            .await
            .map_err(|e| HarnessError::Internal(e.to_string()))?
            .map_err(|_| HarnessError::Internal("yazıcı panikledi".into()))?;
        let failure = failure
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(err) = failure {
            return Err(err);
        }
        let store = store.map_err(|()| HarnessError::Internal("yazıcı durdu".into()))?;

        let total = store.counts()?;
        let summary = RunSummary {
            run_id,
            session: counters.tally(),
            total,
            remaining: self.plan.items.saturating_sub(total.done + total.rejected),
            interrupted: cancel.is_cancelled(),
            reject_reasons: store.reject_reasons()?,
            repaired,
        };
        info!(
            done = summary.total.done,
            rejected = summary.total.rejected,
            failed = summary.total.failed,
            remaining = summary.remaining,
            interrupted = summary.interrupted,
            reasons = ?summary.reject_reasons,
            "çalıştırma bitti"
        );
        Ok(summary)
    }

    /// Plan özetini ve kökeni denetler; çalıştırma kimliğini döner.
    fn check_identity(&self, store: &Store) -> Result<String, HarnessError> {
        match store.meta("plan_hash")? {
            Some(stored) if stored != self.plan.hash() => {
                return Err(HarnessError::PlanChanged {
                    stored,
                    current: self.plan.hash().to_owned(),
                });
            }
            Some(_) => {}
            None => store.set_meta("plan_hash", self.plan.hash())?,
        }
        match store.meta("provenance")? {
            Some(stored) => {
                let stored_value: serde_json::Value =
                    serde_json::from_str(&stored).unwrap_or(serde_json::Value::Null);
                if stored_value != self.provenance {
                    return Err(HarnessError::ProvenanceChanged(format!(
                        "kayıtlı {stored}, bu çalıştırma {}",
                        self.provenance
                    )));
                }
            }
            None => store.set_meta("provenance", &self.provenance.to_string())?,
        }
        if let Some(id) = store.meta("run_id")? {
            return Ok(id);
        }
        let id = new_run_id(&self.plan);
        store.set_meta("run_id", &id)?;
        Ok(id)
    }

    async fn dispatch<S: Stage>(
        &self,
        stage: &Arc<S>,
        finished: &HashSet<String>,
        tx: &mpsc::Sender<Event>,
        stop: &CancellationToken,
        failure: &Abort,
    ) {
        let permits = Arc::new(Semaphore::new(self.plan.concurrency));
        let mut tasks = JoinSet::new();
        for index in 0..self.plan.items {
            let item = self.plan.item(index);
            if finished.contains(&item.id) {
                continue;
            }
            let permit = tokio::select! {
                biased;
                () = stop.cancelled() => break,
                p = Arc::clone(&permits).acquire_owned() => p.expect("semafor kapatılmaz"),
            };
            while let Some(res) = tasks.try_join_next() {
                check_join(res, failure, stop);
            }
            let stage = Arc::clone(stage);
            let tx = tx.clone();
            let stop = stop.clone();
            let failure = Arc::clone(failure);
            let max_attempts = self.plan.max_attempts;
            tasks.spawn(async move {
                let _permit = permit;
                process(&*stage, item, max_attempts, &tx, &stop, &failure).await;
            });
        }
        while let Some(res) = tasks.join_next().await {
            check_join(res, failure, stop);
        }
    }

    fn spawn_progress<S: Stage>(
        &self,
        stage: Arc<S>,
        counters: Arc<Counters>,
    ) -> tokio::task::JoinHandle<()> {
        let every = self.progress_every;
        tokio::spawn(async move {
            let start = Instant::now();
            let mut tick = tokio::time::interval(every);
            tick.tick().await;
            loop {
                tick.tick().await;
                let t = counters.tally();
                let secs = start.elapsed().as_secs_f64().max(1e-9);
                #[allow(clippy::cast_precision_loss)]
                let per_sec = (t.done + t.rejected + t.failed) as f64 / secs;
                let extra: Vec<String> = stage
                    .counters()
                    .into_iter()
                    .map(|(k, v)| {
                        #[allow(clippy::cast_precision_loss)]
                        let rate = v as f64 / secs;
                        format!("{k}={v} ({rate:.1}/sn)")
                    })
                    .collect();
                info!(
                    done = t.done,
                    rejected = t.rejected,
                    failed = t.failed,
                    per_sec = format!("{per_sec:.2}"),
                    counters = extra.join(" "),
                    "ilerleme"
                );
            }
        })
    }
}

fn check_join(res: Result<(), tokio::task::JoinError>, failure: &Abort, stop: &CancellationToken) {
    if let Err(e) = res {
        abort(
            failure,
            stop,
            HarnessError::Internal(format!("görev çöktü: {e}")),
        );
    }
}

async fn process<S: Stage>(
    stage: &S,
    item: Item,
    max_attempts: u32,
    tx: &mpsc::Sender<Event>,
    stop: &CancellationToken,
    failure: &Abort,
) {
    let mut attempt = 0;
    let event = loop {
        match stage.run(&item, attempt).await {
            Ok(output) => {
                let line = serde_json::to_value(&output)
                    .map_err(|e| e.to_string())
                    .and_then(|value| {
                        if !value.is_object() {
                            return Err("çıktı bir JSON nesnesi değil".to_owned());
                        }
                        serde_json::to_vec(&Line {
                            item: &item.id,
                            index: item.index,
                            seed: item.seed,
                            attempt,
                            output: &value,
                        })
                        .map_err(|e| e.to_string())
                    });
                match line {
                    Ok(line) => {
                        break Event::Done {
                            item,
                            attempts: attempt + 1,
                            line,
                        };
                    }
                    Err(message) => {
                        let item = item.id;
                        abort(failure, stop, HarnessError::Fatal { item, message });
                        return;
                    }
                }
            }
            Err(StageError::Reject { reason }) => {
                break Event::Rejected {
                    item,
                    attempts: attempt + 1,
                    reason,
                };
            }
            Err(StageError::Retryable(e)) => {
                if attempt + 1 < max_attempts {
                    if stop.is_cancelled() {
                        // Durduruluyor: örnek bekleyen kalır, sonra yeniden denenir.
                        return;
                    }
                    debug!(item = %item.id, attempt, error = %e, "örnek yeniden denenecek");
                    attempt += 1;
                    continue;
                }
                warn!(item = %item.id, error = %e, "örneğin denemeleri tükendi");
                break Event::Failed {
                    item,
                    attempts: attempt + 1,
                    error: e.to_string(),
                };
            }
            Err(StageError::Fatal(e)) => {
                let item = item.id;
                let message = e.to_string();
                abort(failure, stop, HarnessError::Fatal { item, message });
                return;
            }
        }
    };
    // Yazıcı durduysa hata zaten kaydedildi.
    let _ = tx.send(event).await;
}

/// Tek yazıcı: satırı yazar ve diske indirir, sonra veritabanında işaretler.
fn write_loop(
    mut rx: mpsc::Receiver<Event>,
    store: Store,
    mut writer: shard::Writer,
    counters: &Counters,
) -> Result<Store, HarnessError> {
    while let Some(event) = rx.blocking_recv() {
        match event {
            Event::Done {
                item,
                attempts,
                line,
            } => {
                let shard = writer.append(&line)?;
                store.mark(
                    &item.id,
                    item.index,
                    Status::Done,
                    attempts,
                    None,
                    Some(&shard),
                )?;
                counters.done.fetch_add(1, Ordering::Relaxed);
            }
            Event::Rejected {
                item,
                attempts,
                reason,
            } => {
                store.mark(
                    &item.id,
                    item.index,
                    Status::Rejected,
                    attempts,
                    Some(&reason),
                    None,
                )?;
                counters.rejected.fetch_add(1, Ordering::Relaxed);
            }
            Event::Failed {
                item,
                attempts,
                error,
            } => {
                store.mark(
                    &item.id,
                    item.index,
                    Status::Failed,
                    attempts,
                    Some(&error),
                    None,
                )?;
                counters.failed.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    Ok(store)
}

/// Ctrl-C'de tetiklenen bir durdurma işareti. Tokio çalışma zamanı içinde
/// çağrılmalı.
#[must_use]
pub fn cancel_on_ctrl_c() -> CancellationToken {
    let token = CancellationToken::new();
    let trigger = token.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            warn!("durduruluyor: yeni örnek başlatılmıyor, sürenler bitirilip yazılıyor");
            trigger.cancel();
        }
    });
    token
}

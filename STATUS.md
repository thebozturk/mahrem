# Durum

Bu dosya "nerede kaldık" sorusunun cevabıdır. Kısa kalır ve her iş bitiminde üzerine yazılır; geçmiş `git log`'dadır, büyük plan [ROADMAP.md](ROADMAP.md)'dedir.

Son güncelleme: 2026-11-20

## Aşama

**Hazırlık** (ölçüm ve veri). Çıkış kapısı: model kararı.

## Son biten

- Benchmark CLI: `mahrem-bench` ([docs/benchmark.md](docs/benchmark.md))
  - `score`, `predict`, `validate`, `stats`; metrikler `mahrem-dataset::eval`'de
  - Sızıntısız belge oranı (etiketten bağımsız kaplama), yüksek riskli oran, yanlış alarm, etiket başına korunma ve kesin/örtüşmeli recall-precision; `--scope`
  - Rapor yalnızca sayı içerir (testte); karar [0012](docs/decisions/0012-benchmark-metrikleri.md)
  - Zor örnekler, yapısal kapsam: 27 span'den 25'i korunuyor (kaçanlar yazıyla yazılmış iki numara), yanlış alarm 0; 2.000 karakterde p95 < 1 ms (release)
- Daha önce: yapısal tanıyıcılar (`analyze`), veri üretimi hattı, sahte değer üreticileri, harness ve LLM istemcisi, altın set kompozisyonu, zor örnekler seti, yönerge 0.2.0, altın set şeması

## Sıradaki

1. Üretim modelinin seçimi ve altın set v0 üretimi (ayrı makinede sürüyor)
2. Elle yazılmış belgeler: her bölüm için en az 50 (proje sahibi yazar)
3. İlk açık rapor: altın set v0 gelince `mahrem-bench` ile açık ve kapalı bölüm; bitince ROADMAP'teki "Benchmark CLI ve ilk açık rapor" kutusu işaretlenir
4. v0.1: Python paketinin yayını (`pip install mahrem`)
5. Etiketleme iş akışı: tanıyıcılar, NER ve sözlükler; uyuşmazlıkta yerel LLM; Argilla

## Yarım kalan iş

Altın set v0: üretim hattı hazır, içerik yok. Model seçimi ve elle yazılmış belgeler bekleniyor.

## Açık sorular

- `mahrem-llm` yalnızca `http` konuşuyor. TLS ilk uzak uçla (gateway) gelecek; `reqwest`'in TLS kökleri CDLA-Permissive-2.0 lisanslı, izin listesinde değil.

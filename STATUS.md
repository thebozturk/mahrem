# Durum

Bu dosya "nerede kaldık" sorusunun cevabıdır. Kısa kalır ve her iş bitiminde üzerine yazılır; geçmiş `git log`'dadır, büyük plan [ROADMAP.md](ROADMAP.md)'dedir.

Son güncelleme: 2026-11-22

## Aşama

**Hazırlık** (ölçüm ve veri). Çıkış kapısı: model kararı.

## Son biten

- İlk yayın v0.0.1: [`mahrem`](https://pypi.org/project/mahrem/) PyPI'de, [`mahrem-core`](https://crates.io/crates/mahrem-core) crates.io'da, [GitHub Release](https://github.com/thebozturk/mahrem/releases/tag/v0.0.1)
  - Yayın akışı ([karar 0013](docs/decisions/0013-yayin-akisi.md)) TestPyPI provası ve gerçek yayınla uçtan uca denendi
  - Sonraki sürümler yalnızca etiketle çıkar; adımlar CONTRIBUTING'de
- Daha önce: benchmark CLI (`mahrem-bench`), yapısal tanıyıcılar (`analyze`), veri üretimi hattı, sahte değer üreticileri, harness ve LLM istemcisi, altın set kompozisyonu, zor örnekler seti, yönerge 0.2.0, altın set şeması

## Sıradaki

1. Üretim modelinin seçimi ve altın set v0 üretimi (ayrı makinede sürüyor)
2. Elle yazılmış belgeler: her bölüm için en az 50 (proje sahibi yazar)
3. İlk açık rapor: altın set v0 gelince `mahrem-bench` ile açık ve kapalı bölüm
4. Etiketleme iş akışı: tanıyıcılar, NER ve sözlükler; uyuşmazlıkta yerel LLM; Argilla

## Yarım kalan iş

Altın set v0: üretim hattı hazır, içerik yok. Model seçimi ve elle yazılmış belgeler bekleniyor.

## Açık sorular

- `mahrem-llm` yalnızca `http` konuşuyor. TLS ilk uzak uçla (gateway) gelecek; `reqwest`'in TLS kökleri CDLA-Permissive-2.0 lisanslı, izin listesinde değil.

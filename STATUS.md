# Durum

Bu dosya "nerede kaldık" sorusunun cevabıdır. Kısa kalır ve her iş bitiminde üzerine yazılır; geçmiş `git log`'dadır, büyük plan [ROADMAP.md](ROADMAP.md)'dedir.

Son güncelleme: 2026-11-20

## Aşama

**Hazırlık** (ölçüm ve veri). Çıkış kapısı: model kararı.

## Son biten

- Yayın akışı: `.github/workflows/release.yml`, karar [0013](docs/decisions/0013-yayin-akisi.md)
  - `v*` etiketiyle: wheel matrisi (Linux manylinux/musllinux, macOS, Windows; abi3) ve sdist, kurulu paketle test, onaylı ortamlarda PyPI ve crates.io'ya trusted publishing, GitHub Release
  - Elle tetikleme TestPyPI'ye prova yükler
  - Paket README'leri (`mahrem`, `mahrem-core`), `CHANGELOG.md`, CONTRIBUTING'de sürüm çıkarma
- Daha önce: benchmark CLI (`mahrem-bench`), yapısal tanıyıcılar (`analyze`), veri üretimi hattı, sahte değer üreticileri, harness ve LLM istemcisi, altın set kompozisyonu, zor örnekler seti, yönerge 0.2.0, altın set şeması

## Sıradaki

1. İlk yayın v0.0.1 (proje sahibi): PyPI/TestPyPI pending publisher, GitHub ortamları, `mahrem-core`'un ilk elle yayını, TestPyPI provası, etiket. Bitince ROADMAP'teki "Rust çekirdeği ve Python paketi" kutusu işaretlenir
2. Üretim modelinin seçimi ve altın set v0 üretimi (ayrı makinede sürüyor)
3. Elle yazılmış belgeler: her bölüm için en az 50 (proje sahibi yazar)
4. İlk açık rapor: altın set v0 gelince `mahrem-bench` ile açık ve kapalı bölüm
5. Etiketleme iş akışı: tanıyıcılar, NER ve sözlükler; uyuşmazlıkta yerel LLM; Argilla

## Yarım kalan iş

Altın set v0: üretim hattı hazır, içerik yok. Model seçimi ve elle yazılmış belgeler bekleniyor.

## Açık sorular

- `mahrem-llm` yalnızca `http` konuşuyor. TLS ilk uzak uçla (gateway) gelecek; `reqwest`'in TLS kökleri CDLA-Permissive-2.0 lisanslı, izin listesinde değil.

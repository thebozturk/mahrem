# 0013: Yayın akışı: etiketle tetiklenir, onaylı ortamda, anahtarsız

Durum: kabul edildi
Tarih: 2026-11-20

## Bağlam

Python paketi PyPI'ye, çekirdek crates.io'ya yayınlanacak. Mart 2026'da PyPI'deki LiteLLM paketine zararlı sürümler yüklendi. Mahrem bir güvenlik aracı olduğu için yayın yolu da saldırı yüzeyidir. PRD şunları istiyor: abi3 wheel'ler (macOS, Linux, Windows), Python 3.10 ve üstü, trusted publishing, imza, semver. Adların ilk çalışan sürümle alınması ve boş yer tutucu paket yayınlanmaması da PRD'de yazılı.

## Karar

- **Yayın yalnızca `v*` etiketiyle başlar.** Etiket `Cargo.toml` sürümüyle ve `CHANGELOG.md`'deki bir başlıkla eşleşmezse iş akışı durur.
- **Wheel matrisi:** Linux manylinux ve musllinux (x86_64, aarch64), macOS (x86_64, arm64), Windows x64 ve sdist. abi3 sayesinde her platformda tek wheel yeter.
- **Kurulu paketle test:** Yayınlanacak wheel temiz ortama kurulur ve conformance testleri onunla çalışır. Python 3.10 ve 3.14'te; Linux x86_64 ve arm64, macOS ve Windows'ta.
- **Anahtarsız yayın:**
  - PyPI ve crates.io'ya trusted publishing (OIDC) ile yayınlanır; repoda hiçbir anahtar tutulmaz.
  - PyPI yüklemesi Sigstore tabanlı attestation (PEP 740) üretir. Wheel'ler maturin'in gömdüğü CycloneDX SBOM'u taşır.
- **Onaylı ortamlar:** Yayın adımları `pypi`, `crates-io` ve `testpypi` ortamlarında çalışır ve proje sahibinin onayını bekler. Etiket push edilse de onaysız bir şey yayınlanmaz.
- **Prova:** Elle tetiklenen çalıştırma aynı derlemeyi TestPyPI'ye yükler.
- **İzinler:** Action'lar SHA'ya sabittir. İzinler iş başına en dar haliyle verilir; `id-token: write` yalnızca yayın işlerinde. Yayın işinde önbellek kapalıdır (önbellek zehirlenmesi).
- **İlk sürüm v0.0.1.** PyPI'de `mahrem`, crates.io'da `mahrem-core`. v0.1 adı Kapı 1'e kalır.

## Reddedilenler

- **Repoda saklanan PyPI ya da crates.io anahtarı:** Sızarsa herkes sürüm yayınlayabilir.
- **Main'e her birleşmede otomatik yayın:** Sürüm bilinçli bir karar olmalı.
- **v0.1.0 ile başlamak:** Kapı 1 geçilmeden "v0.1 bitti" izlenimi verir.

## Sonuç

crates.io trusted publishing'i bir crate'in ilk sürümü için desteklemiyor. `mahrem-core` v0.0.1 bir kez elle yayınlanır; sonraki sürümleri iş akışı yayınlar. Sürüm zaten yayınlıysa iş akışı crates.io adımını atlar. Gateway ve benchmark binary'lerinin yayını (statik derleme, cargo-auditable, Docker imajı) ayrı bir iş.

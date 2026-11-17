# Ajan kılavuzu

Bu dosya, repoda çalışan yapay zekâ ajanları (Claude Code, Codex, Cursor ve benzerleri) içindir. Her oturumun başında okunur; bu yüzden kısa tutulur. Claude Code bu dosyayı ve [STATUS.md](STATUS.md)'yi [CLAUDE.md](CLAUDE.md) üzerinden kendiliğinden yükler.

## Proje

Mahrem, Türkçe metindeki kişisel verileri LLM'e gitmeden önce maskeler ve yanıtta gerçek değerleri doğru Türkçe ekle geri yerleştirir. Tek bir Rust çekirdeği vardır; Python paketi, gateway ve benchmark aynı çekirdeği çalıştırır. Proje takvimle değil ölçüm kapılarıyla ilerler.

## Hangi bilgi nerede

Her bilgi tek bir dosyada durur. Aynı şeyi ikinci bir dosyaya yazma, bağlantı ver.

| Soru | Dosya |
| --- | --- |
| Proje ne yapar, tasarım nasıl? | [README.md](README.md) |
| Şu an neredeyiz, sırada ne var? | [STATUS.md](STATUS.md) |
| Büyük plan ve kapılar | [ROADMAP.md](ROADMAP.md) |
| Crate sınırları ve bağımlılık yönü | [docs/architecture.md](docs/architecture.md) |
| Altın set ve tahmin dosyasının formatı, etiket listesi | [docs/gold-set/](docs/gold-set/README.md) |
| Neden böyle yaptık? | [docs/decisions/](docs/decisions/README.md) |
| Katkı kuralları | [CONTRIBUTING.md](CONTRIBUTING.md) |

## Harita

| Yol | İş |
| --- | --- |
| `crates/mahrem-core` | Çekirdek: tespit, maskeleme, geri yerleştirme |
| `crates/mahrem-py` | Python bağlaması (PyO3); Python API'si `python/mahrem/` altında |
| `crates/mahrem-dataset` | Altın set ve tahmin dosyalarının kayıt tipleri ve doğrulayıcısı; format: [docs/gold-set/](docs/gold-set/README.md) |
| `crates/mahrem-ner` | Türkçe NER (ONNX Runtime, CPU) |
| `crates/mahrem-vault` | Etiket ile gerçek değer eşlemesinin şifreli saklanması |
| `crates/mahrem-llm` | OpenAI uyumlu istemci; gateway ve harness ortak kullanır |
| `crates/mahrem-harness` | Kaldığı yerden devam eden toplu iş çalıştırıcı |
| `crates/mahrem-tools` | Ajanların ve MCP sunucusunun ortak araç tanımları |
| `crates/mahrem-gateway` | LLM gateway (binary) |
| `crates/mahrem-bench` | Benchmark CLI (binary) |
| `conformance/` | Rust ve Python'un birlikte okuduğu ortak test vakaları |

Hangi crate'in dolu, hangisinin yer tutucu olduğu README'deki "Repository layout" tablosundadır.

## Komutlar

```sh
make check      # rustfmt + clippy (-D warnings) + testler
make py-test    # Python paketi: uv sync, pytest, ruff
make deny       # lisans, kaynak ve güvenlik denetimi (cargo-deny)
```

Rust sürümü `rust-toolchain.toml` ile sabittir. Python tarafı `uv` ister.

## Bozulmayacak kurallar

Ayrıntı ve gerekçe [CONTRIBUTING.md](CONTRIBUTING.md)'dedir.

- **Gerçek kişisel veri yok.** Test, örnek, issue, PR ve commit mesajında yalnızca sentetik değer kullanılır.
- **Repo herkese açıktır.** Anahtar, müşteri adı ya da özel not commit'lenmez. Özel notlar `internal/`, veri `data/` altında durur; ikisi de `.gitignore`'dadır.
- **`unsafe` yasaktır.** Tek istisna `mahrem-py`.
- **Bağımlılık:** yalnızca izinli lisans, yalnızca crates.io, git bağımlılığı yok.
- **Terim:** Mahrem "anonimleştirme aracı" değildir; geri çevrilebilen maskeleme takma adlandırmadır.
- **Conformance:** yeni bir doğrulayıcı ya da dönüşüm eklenince `conformance/` setine vaka eklenir, Rust ve Python testleri birlikte güncellenir.
- **Commit:** DCO imzalı (`git commit -s`).

## Oturum akışı

1. Başlarken [STATUS.md](STATUS.md)'yi oku. Sıradaki iş oradadır.
2. Bitirmeden önce `make check` (Python değiştiyse `make py-test`) geçmeli.
3. İş bitince, aynı commit ya da aynı PR içinde:
   - [STATUS.md](STATUS.md)'yi güncelle. Üzerine yaz, günlük gibi alta ekleme.
   - Bir [ROADMAP.md](ROADMAP.md) maddesi bittiyse kutusunu işaretle; README'deki durum tablosu değiştiyse onu da düzelt.
   - Kalıcı bir tasarım kararı verildiyse [docs/decisions/](docs/decisions/README.md) altına kısa bir kayıt ekle.
4. Bir yolu önermeden önce [docs/decisions/](docs/decisions/README.md)'a bak; daha önce bilerek reddedilmiş olabilir.

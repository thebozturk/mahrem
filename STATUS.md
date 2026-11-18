# Durum

Bu dosya "nerede kaldık" sorusunun cevabıdır. Kısa kalır ve her iş bitiminde üzerine yazılır; geçmiş `git log`'dadır, büyük plan [ROADMAP.md](ROADMAP.md)'dedir.

Son güncelleme: 2026-11-18

## Aşama

**Hazırlık** (ölçüm ve veri). Çıkış kapısı: model kararı.

## Son biten

- Harness ve LLM istemcisi
  - `mahrem-harness`: TOML plan, `Stage` trait'i, append-only JSONL shard'lar ve SQLite durumu. Kaldığı yerden devam eder; süreç rastgele anlarda öldürülse de kayıp ya da çift örnek olmuyor (testte). Köken `run.json`'da. Karar [0008](docs/decisions/0008-harness.md)
  - `mahrem-llm`: OpenAI uyumlu, akışsız istemci; yeniden deneme, `Retry-After`, eşzamanlılık sınırı, token sayacı. Varsayılan olarak yalnızca yerel uç; karar [0009](docs/decisions/0009-llm-yerel-uc.md)
  - En düşük Rust sürümü 1.89 (`rusqlite` 0.41 için)
- Daha önce: altın set kompozisyonu ve araçları, zor örnekler seti (82 belge), etiketleme yönergesi 0.2.0, altın set şeması

## Sıradaki

Sıra karar 0007'ye göre: altın setin içeriği yerel üretime bağlı.

1. Veri üretimi (`mahrem-datagen`, yerel Gemma): şablon ve yer tutuculu metin, değer üreticileri, tekilleştirme. Harness ve LLM istemcisini birleştiren ilk binary
2. Altın set v0'ın içeriği: üretim, bölme, insan onayı. Bitince ROADMAP'teki "Altın set v0" kutusu işaretlenir
3. Etiketleme iş akışı: tanıyıcılar, NER ve sözlükler; uyuşmazlıkta yerel LLM; Argilla

## Yarım kalan iş

Altın set v0: kompozisyon ve araçlar hazır, içerik yok (yukarıda 2. madde).

## Açık sorular

- Doğrulayıcının ve dağılım raporunun komut satırı arayüzü yok; `mahrem-bench` gelince eklenir.
- `mahrem-llm` yalnızca `http` konuşuyor. TLS ilk uzak uçla (gateway) gelecek; `reqwest`'in TLS kökleri CDLA-Permissive-2.0 lisanslı, izin listesinde değil.

# 0006: Açık veri setleri repoda `datasets/` altında; köken her kayıtta yazılı

Durum: kabul edildi
Tarih: 2026-11-18

## Bağlam

Zor örnekler seti, ilk açık veri setidir; ardından altın setin açık bölümü gelecek. Bu setlerin nerede durduğu ve metni kimin yazdığının nasıl kaydedildiği belirlenmeliydi. Zor örnekler setinin metinlerini bir LLM (Claude) yazdı.

## Karar

- Açık veri setleri repoda `datasets/` altında durur, CC BY 4.0'dır (`datasets/LICENSE`). Sürümlerde Hugging Face'e (`mahrem-ai`) yayınlanır.
- CI, `datasets/` altındaki tüm JSONL dosyalarını birlikte doğrular: hata da uyarı da kabul edilmez. Kapalı bölüm dosyası (`*.closed.jsonl`) bu klasörde bulunamaz.
- Metni yazan, kaydın `meta.source` ve `meta.origin.generator` alanlarında olduğu gibi yazılır. LLM'in yazdığı metin `source: "llm"` ve model adını taşır.
- Altın sete yalnızca insan onaylı etiket girer. Onay, pull request incelemesidir.

## Reddedilenler

- **Yalnızca Hugging Face'te tutmak:** Veri, yönerge ve doğrulayıcı ayrı yerlerde değişir; CI doğrulaması için indirme gerekir.
- **`data/` klasörü:** `.gitignore`'da; özel ve kapalı veri için ayrılmış.
- **LLM'in yazdığı metni "handwritten" kaydetmek:** Köken kaydı yanlış olur; dışarıdan bakan, verinin nasıl üretildiğini göremez.

## Sonuç

Yönerge, veri ve doğrulayıcı aynı pull request'te birlikte değişir. Repo büyüdükçe açık bölümün boyutu izlenmeli; çok büyürse yalnızca Hugging Face'e taşımak yeni bir karar gerektirir.

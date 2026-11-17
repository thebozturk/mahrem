# Durum

Bu dosya "nerede kaldık" sorusunun cevabıdır. Kısa kalır ve her iş bitiminde üzerine yazılır; geçmiş `git log`'dadır, büyük plan [ROADMAP.md](ROADMAP.md)'dedir.

Son güncelleme: 2026-11-18

## Aşama

**Hazırlık** (ölçüm ve veri). Çıkış kapısı: model kararı.

## Son biten

- Altın set şeması (schema_version 1): [docs/gold-set/](docs/gold-set/README.md)
  - 31 etiket (`Label`, `mahrem-core`), KVKK m.6/1 on etikette; çocuk verisi `child` işareti
  - Konumlar code point, bitiş hariç, NFC; Rust dönüşümü `CodePointIndex`
  - Kayıt tipleri, tahmin formatı ve doğrulayıcı: yeni `mahrem-dataset` crate'i
  - Kararlar: [0002](docs/decisions/0002-konum-birimi.md), [0003](docs/decisions/0003-etiket-kimlikleri.md), [0004](docs/decisions/0004-veri-seti-crate.md)
- CI düzeltmesi: ruff 0.17 Markdown içindeki Python bloklarını da biçimliyor; README örneği biçimlendi

## Sıradaki

Altın set v0'ın içeriği ve etiketleme yönergesi şemaya bağlı olarak başlayabilir. ROADMAP'teki "Altın set v0" kutusu içerik gelince işaretlenir.

1. Etiketleme yönergesi ve zor örnekler seti (Türkçe; şemadaki `guideline_version` buna bağlanır)
2. Altın set v0: açık ve kapalı bölüm
3. Rust harness ve LLM istemcisi (`mahrem-harness`, `mahrem-llm`)

## Yarım kalan iş

Yok.

## Açık sorular

- Etiketleme yönergesi şunları netleştirmeli: yaş span'i ("[7 yaş]ında" mı, "[7]" mi), tek başına il adı etiketlenir mi, `ORGANIZATION` her kurum mu yoksa yalnızca kişiyle ilişkili olanlar mı.
- Doğrulayıcının komut satırı arayüzü yok; `mahrem-bench` gelince eklenir.

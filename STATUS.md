# Durum

Bu dosya "nerede kaldık" sorusunun cevabıdır. Kısa kalır ve her iş bitiminde üzerine yazılır; geçmiş `git log`'dadır, büyük plan [ROADMAP.md](ROADMAP.md)'dedir.

Son güncelleme: 2026-11-18

## Aşama

**Hazırlık** (ölçüm ve veri). Çıkış kapısı: model kararı.

## Son biten

- Altın set v0, 1. kısım: kompozisyon ve araçlar. İçerik henüz yok
  - [docs/gold-set/composition.md](docs/gold-set/composition.md) ve hedefler: [composition-v0.json](docs/gold-set/composition-v0.json) (bölüm başına en az 500 belge)
  - `mahrem-dataset`: dağılım raporu (`Stats`, `Composition::check`), grup düzeyinde bölme (`part_of`), kapalı bölüm özeti (`digest`); Python ile aynı sonucu verdiği testte
  - Kayıtlara `origin.group` alanı eklendi; karar [0007](docs/decisions/0007-altin-set-kompozisyonu.md)
- Daha önce: zor örnekler seti (82 belge), etiketleme yönergesi 0.2.0, altın set şeması

## Sıradaki

Sıra karar 0007'ye göre: altın setin içeriği yerel üretime bağlı.

1. Rust harness ve LLM istemcisi (`mahrem-harness`, `mahrem-llm`)
2. Veri üretimi (`mahrem-datagen`, yerel Gemma): şablon ve yer tutuculu metin, değer üreticileri, tekilleştirme
3. Altın set v0'ın içeriği: üretim, bölme, insan onayı. Bitince ROADMAP'teki "Altın set v0" kutusu işaretlenir
4. Etiketleme iş akışı: tanıyıcılar, NER ve sözlükler; uyuşmazlıkta yerel LLM; Argilla

## Yarım kalan iş

Altın set v0: kompozisyon ve araçlar hazır, içerik yok (yukarıda 3. madde).

## Açık sorular

- Doğrulayıcının ve dağılım raporunun komut satırı arayüzü yok; `mahrem-bench` gelince eklenir.

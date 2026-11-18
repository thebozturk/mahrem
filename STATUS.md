# Durum

Bu dosya "nerede kaldık" sorusunun cevabıdır. Kısa kalır ve her iş bitiminde üzerine yazılır; geçmiş `git log`'dadır, büyük plan [ROADMAP.md](ROADMAP.md)'dedir.

Son güncelleme: 2026-11-18

## Aşama

**Hazırlık** (ölçüm ve veri). Çıkış kapısı: model kararı.

## Son biten

- Veri üretimi hattı: `mahrem-datagen` ([kılavuz](docs/datagen.md))
  - Belge planı kompozisyon hedeflerini tutturuyor (testte); 8 metin türünde 88 senaryo, senaryo bölme grubudur
  - Yer tutucu denetimi, yer tutucu dışında kalmış veri araması, ekle uyumlu değer seçimi, zor durum dönüşümleri, tekilleştirme, bölme ve rapor
  - `fill` (elle yazılmış metinler) ve `blind` (model seçimi için kör okuma) komutları
  - Sahte LLM sunucusuyla uçtan uca testte
- Daha önce: sahte değer üreticileri, harness ve LLM istemcisi, altın set kompozisyonu, zor örnekler seti, etiketleme yönergesi 0.2.0, altın set şeması

## Sıradaki

1. Üretim modelinin seçimi: iki adayla yaklaşık 50'şer örnek, ret oranı ve kör okuma
2. Elle yazılmış belgeler: her bölüm için en az 50 (proje sahibi yazar)
3. Altın set v0 üretimi, `build`, insan onayı; açık bölüm `datasets/` altına. Bitince ROADMAP'teki "Altın set v0" kutusu işaretlenir
4. Etiketleme iş akışı: tanıyıcılar, NER ve sözlükler; uyuşmazlıkta yerel LLM; Argilla

## Yarım kalan iş

Altın set v0: üretim hattı hazır, içerik yok. Model seçimi ve elle yazılmış belgeler bekleniyor.

## Açık sorular

- Doğrulayıcının ve dağılım raporunun komut satırı arayüzü yok; `mahrem-bench` gelince eklenir.
- `mahrem-llm` yalnızca `http` konuşuyor. TLS ilk uzak uçla (gateway) gelecek; `reqwest`'in TLS kökleri CDLA-Permissive-2.0 lisanslı, izin listesinde değil.

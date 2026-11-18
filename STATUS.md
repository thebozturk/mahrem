# Durum

Bu dosya "nerede kaldık" sorusunun cevabıdır. Kısa kalır ve her iş bitiminde üzerine yazılır; geçmiş `git log`'dadır, büyük plan [ROADMAP.md](ROADMAP.md)'dedir.

Son güncelleme: 2026-11-18

## Aşama

**Hazırlık** (ölçüm ve veri). Çıkış kapısı: model kararı.

## Son biten

- Sahte değer üreticileri (`mahrem_core::fake`), veri üretiminin 1. kısmı
  - Her etiket için tohumlu üretici. Kimlik ve finans numaraları checksum'ı doğru; her değer altın set doğrulayıcısından uyarısız geçiyor (testte)
  - Ad, soyad, il, ilçe, cadde ve kurum listeleri bu projede yazıldı; KVKK m.6 değer sözcükleri; sayıların Türkçe yazımı
  - Karar [0010](docs/decisions/0010-sahte-deger-ureticileri.md)
- Daha önce: harness ve LLM istemcisi, altın set kompozisyonu ve araçları, zor örnekler seti (82 belge), etiketleme yönergesi 0.2.0, altın set şeması

## Sıradaki

Sıra karar 0007'ye göre: altın setin içeriği yerel üretime bağlı.

1. Veri üretimi, 2. kısım: `mahrem-datagen` binary'si (belge planı, prompt'lar, doğrulama, doldurma ve ek uyumu, zor durum dönüşümleri, tekilleştirme, bölme). Model ayrı makinede (M1 Pro, 16 GB) yerel çalışır; datagen de orada çalışır
2. Altın set v0'ın içeriği: üretim, bölme, insan onayı. Bitince ROADMAP'teki "Altın set v0" kutusu işaretlenir
3. Etiketleme iş akışı: tanıyıcılar, NER ve sözlükler; uyuşmazlıkta yerel LLM; Argilla

## Yarım kalan iş

Altın set v0: kompozisyon ve araçlar hazır, içerik yok (yukarıda 2. madde).

## Açık sorular

- Doğrulayıcının ve dağılım raporunun komut satırı arayüzü yok; `mahrem-bench` gelince eklenir.
- `mahrem-llm` yalnızca `http` konuşuyor. TLS ilk uzak uçla (gateway) gelecek; `reqwest`'in TLS kökleri CDLA-Permissive-2.0 lisanslı, izin listesinde değil.

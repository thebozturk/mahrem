# Durum

Bu dosya "nerede kaldık" sorusunun cevabıdır. Kısa kalır ve her iş bitiminde üzerine yazılır; geçmiş `git log`'dadır, büyük plan [ROADMAP.md](ROADMAP.md)'dedir.

Son güncelleme: 2026-11-20

## Aşama

**Hazırlık** (ölçüm ve veri). Çıkış kapısı: model kararı.

## Son biten

- v0.1 yapısal tanıyıcılar: `analyze` ([docs/detection.md](docs/detection.md))
  - TCKN/YKN, VKN, IBAN, kart, telefon ve e-posta; boşluklu, tireli, satıra bölünmüş numaralar ve "at/nokta"lı e-posta
  - Skor ve bağlam sözcükleri; bağlamsız VKN varsayılan eşiğin altında
  - Rust ve Python'da aynı sonuç (conformance); zor örnekler setinde kapsamdaki 25 yapısal span'in hepsi bulunuyor, yanlış alarm yok
  - Karar [0011](docs/decisions/0011-yapisal-tarayici.md)
- Daha önce: veri üretimi hattı (`mahrem-datagen`), sahte değer üreticileri, harness ve LLM istemcisi, altın set kompozisyonu, zor örnekler seti, yönerge 0.2.0, altın set şeması

## Sıradaki

1. Üretim modelinin seçimi ve altın set v0 üretimi (ayrı makinede sürüyor)
2. Elle yazılmış belgeler: her bölüm için en az 50 (proje sahibi yazar)
3. v0.1: benchmark CLI (`mahrem-bench`): bir aracın çıktısını altın sete karşı belge ve kategori bazında ölçer
4. v0.1: Python paketinin yayını (`pip install mahrem`)
5. Etiketleme iş akışı: tanıyıcılar, NER ve sözlükler; uyuşmazlıkta yerel LLM; Argilla

## Yarım kalan iş

Altın set v0: üretim hattı hazır, içerik yok. Model seçimi ve elle yazılmış belgeler bekleniyor.

## Açık sorular

- Doğrulayıcının ve dağılım raporunun komut satırı arayüzü yok; `mahrem-bench` gelince eklenir.
- `mahrem-llm` yalnızca `http` konuşuyor. TLS ilk uzak uçla (gateway) gelecek; `reqwest`'in TLS kökleri CDLA-Permissive-2.0 lisanslı, izin listesinde değil.

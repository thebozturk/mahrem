# 0002: Span konumları code point cinsinden, bitiş hariç

Durum: kabul edildi
Tarih: 2026-11-18

## Bağlam

Altın set, tahmin dosyaları ve benchmark aynı span konumlarını okuyacak. Konumun birimi dillere göre farklı: Rust `str` UTF-8 baytla, Python `str` code point'le, JavaScript UTF-16 birimiyle dilimlenir. Türkçe metinde `ş`, `ğ`, `İ` gibi harfler UTF-8'de iki bayt olduğu için bayt ve karakter konumu hemen ayrışır.

## Karar

- Konumlar **Unicode code point** cinsindendir, **bitiş hariçtir** (`text[start:end]`).
- Metin **NFC** olmak zorundadır; doğrulayıcı NFC olmayan metni reddeder.
- Rust tarafında dönüşümü `mahrem-core`'daki `CodePointIndex` yapar.

## Reddedilenler

- **UTF-8 bayt konumu:** Rust'ta dönüşüm gerekmezdi, ama veri setini en çok Python ve Hugging Face kullanıcıları okuyacak. Onlar için her span'de dönüşüm gerekir ve hata kaynağı olur.
- **UTF-16 birimi:** Yalnızca JavaScript'e uyar.
- **Grafem kümesi:** Kullanıcının gördüğü karaktere en yakın birim, ama Unicode sürümüne bağlı ve hiçbir standart araç bu birimle dilimlemez.
- **Normalleştirmeyi zorunlu tutmamak:** Aynı görünen metin NFC ve NFD'de farklı uzunlukta olur; konumlar kayar.

## Sonuç

Python'da `text[start:end] == span["text"]` dönüşümsüz çalışır; spaCy ve HF `datasets` ile aynıdır. Rust tarafı her dilimlemede tablo kurmak zorunda kalır. JavaScript kullanıcılarının BMP dışı karakterlere (emoji) dikkat etmesi gerekir; şema belgesinde yazılıdır.

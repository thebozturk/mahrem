# Kararlar

Kalıcı tasarım kararlarının kısa kayıtları. Amaç, yeni bir oturumda (insan ya da ajan) aynı tartışmanın baştan yapılmasını ve bilerek reddedilmiş bir yolun yeniden önerilmesini önlemek.

## Ne zaman kayıt açılır?

- Birden fazla makul yol vardı ve biri seçildi.
- Bir yol denendi ya da tartışıldı ve reddedildi.
- Kodu okuyan birinin "neden böyle?" diye soracağı bir şey yapıldı.

Küçük uygulama ayrıntıları için kayıt açılmaz; onlar kod yorumuna yazılır.

## Kurallar

- Dosya adı: `NNNN-kisa-baslik.md`, sıra numarasıyla (`0002-...`).
- Kayıt silinmez ve yeniden yazılmaz. Karar değişirse yeni kayıt açılır, eskisinin durumu "yerini NNNN aldı" yapılır.
- Yarım sayfayı geçmez.

## Şablon

```markdown
# NNNN: Başlık

Durum: kabul edildi | yerini NNNN aldı
Tarih: YYYY-AA-GG

## Bağlam

Hangi sorun vardı?

## Karar

Ne seçildi?

## Reddedilenler

Hangi yollar neden seçilmedi?

## Sonuç

Bu karar neyi kolaylaştırır, neyi zorlaştırır?
```

## Liste

| No | Karar |
| --- | --- |
| [0001](0001-ajan-baglam-dosyalari.md) | Ajan bağlamı repodaki dosyalarda tutulur |
| [0002](0002-konum-birimi.md) | Span konumları code point cinsinden, bitiş hariç |
| [0003](0003-etiket-kimlikleri.md) | Etiket kimlikleri, KVKK m.6 gruplaması ve çocuk verisi |
| [0004](0004-veri-seti-crate.md) | Etiketler çekirdekte, kayıt tipleri `mahrem-dataset`'te |
| [0005](0005-account-id-etiketi.md) | Kuruma özgü kişi numaraları için `ACCOUNT_ID` etiketi |
| [0006](0006-acik-veri-setleri.md) | Açık veri setleri repoda `datasets/` altında; köken her kayıtta yazılı |
| [0007](0007-altin-set-kompozisyonu.md) | Altın setin iki bölümü aynı üretimden çıkar ve grup düzeyinde bölünür |
| [0008](0008-harness.md) | Harness LLM'den bağımsızdır; bitmiş örneklerin kaynağı shard'lardır |
| [0009](0009-llm-yerel-uc.md) | LLM istemcisi varsayılan olarak yalnızca yerel uca bağlanır |

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

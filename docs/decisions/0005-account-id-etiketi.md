# 0005: Kuruma özgü kişi numaraları için `ACCOUNT_ID` etiketi

Durum: kabul edildi
Tarih: 2026-11-18

## Bağlam

Şema 1'in ilk etiket listesinde ([0003](0003-etiket-kimlikleri.md)) müşteri, abone, poliçe ya da hasta numarası gibi, bir kurumun kişiyi tanımak için verdiği numaralara karşılık gelen bir etiket yoktu. Bu numaralar müşteri mesajlarında sık geçer ve kurumun sistemlerinde kişiyi doğrudan tanımlar. Etiketlenmezlerse sızıntı ölçülemez.

## Karar

- Yeni etiket: `ACCOUNT_ID`, placeholder adı `KAYIT_NO`.
- Kapsam: müşteri, abone, poliçe, hasta (protokol), öğrenci, personel sicil, üyelik ve IBAN olmayan banka hesap numarası.
- Kapsam dışı: sipariş, kargo takip, fatura, şikâyet ve talep numaraları. Bunlar kişiyi değil işlemi tanımlar.
- Yüksek riskli grupta değildir. Etiket şema 1'e, yayınlanmış veri olmadığı için şema sürümü değişmeden eklendi.

## Reddedilenler

- **`SERIAL_NUMBER`'ı genişletmek:** Cihaz numarasıyla kişi numarasını tek etikette toplar; kategori bazındaki ölçüm anlamını yitirir.
- **Her kurum türü için ayrı etiket:** Numaranın hangi kuruma ait olduğu metinden çoğu zaman anlaşılmaz; etiketleyici uyumu düşer.
- **Etiketlememek:** Kişiyi tanımlayan bir değer ölçüm dışında kalır.

## Sonuç

Etiket sayısı 32 oldu. Bu numaraların biçimi kuruma göre değiştiği için kural katmanı tek başına yakalayamaz; tespit bağlam sözcüklerine ("müşteri numaram", "poliçe no") ve modele dayanır.

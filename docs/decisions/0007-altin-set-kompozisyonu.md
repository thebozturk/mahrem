# 0007: Altın setin iki bölümü aynı üretimden çıkar ve grup düzeyinde bölünür

Durum: kabul edildi
Tarih: 2026-11-18

## Bağlam

Altın setin açık ve kapalı bölümü "aynı dağılım" olmalı; her sürüm raporu iki sayıyı birlikte verir. Kapalı bölümün metni hiçbir dış servise gidemez. Zor örnekler setinin metinlerini dış bir LLM yazdı.

## Karar

- **İki bölüm aynı süreçle, aynı çalıştırmada üretilir ve sonra bölünür.** Süreç yereldir: elle yazım, şablon ya da yerel açık ağırlıklı model. Bu yüzden içerik harness ve veri üretiminden sonra gelir.
- **Başka amaçla üretilen setler altın sete girmez.** Zor örnekler seti ayrı kalır.
- **v0, bölüm başına en az 500 belgedir.** Kapı 2 ve v1.0 ölçümlerinden önce 1000'e çıkar. Hedefler `docs/gold-set/composition-v0.json`'dadır.
- **Bölme tohumlu ve grup düzeyindedir.** `SHA-256(tohum + "\n" + grup)` özetinden hesaplanır. Aynı şablon ya da prompt'tan çıkan belgeler `meta.origin.group` taşır ve aynı bölüme düşer.
- **Kapalı bölümün özeti kanonik biçim üzerinden alınır:** kimliğe göre sıralı, anahtarları sıralı, boşluksuz JSON satırları. Her dilde yeniden üretilebilir; Rust ve Python'un aynı sonucu verdiği testte doğrulanır.

## Reddedilenler

- **Açık bölümü Claude'a yazdırıp kapalıyı yerelde üretmek:** İki bölüm farklı dağılımlardan gelir, iki sayı kıyaslanamaz.
- **Belge düzeyinde rastgele bölme:** Aynı şablondan çıkan belgeler iki bölüme dağılır; açık bölüm kapalının şablonlarını ele verir.
- **Rastgele sayı üreteciyle bölme:** Sonuç kütüphane sürümüne bağlı olur, başka dilde yeniden üretilemez.
- **Dosyanın ham baytlarının özeti:** Biçim, boşluk ya da sıra değişikliği özeti değiştirir; aynı içerik farklı özet verir.

## Sonuç

Altın set v0'ın içeriği harness ve veri üretimini bekler. Kayıtlara isteğe bağlı `origin.group` alanı eklendi. Eski dosyalar geçerli kaldığı için şema sürümü değişmedi.

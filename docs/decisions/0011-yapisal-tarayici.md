# 0011: Yapısal tanıyıcılar regex yerine elle yazılmış bir tarayıcıyla

Durum: kabul edildi
Tarih: 2026-11-20

## Bağlam

v0.1 altı yapısal veriyi bulur: TCKN/YKN, VKN, IBAN, kart, telefon ve e-posta. Numaralar boşluk, tire, nokta, parantez ve satır sonuyla bölünmüş yazılabilir. Yan yana numaralar birbirine karışmamalı; kodların (`TK000111222`) içindeki rakamlar numara sayılmamalı. PRD çekirdek için `regex` crate'ini öneriyordu.

## Karar

- **Elle yazılmış, tek geçişli tarayıcı.**
  - İzinli ayraçlarla birleşen rakam grupları bir aday dizi olur.
  - Dizinin içinde grup sınırlarında başlayıp biten her pencere tanıyıcılara sorulur: uzunluk, önek, checksum.
  - Ayraç kuralı tek yerdedir.
  - Çekirdek bağımlılıksız kalır; tarama doğrusal zamanlıdır (pencere en çok 24 hane).
- **Skor ve bağlam:** checksum'lı değerler yüksek skor alır, bağlam sözcükleri skoru yükseltir. Rastgele 10 haneli sayıların yaklaşık onda biri VKN checksum'ından geçtiği için bağlamsız VKN 0,4 alır ve varsayılan eşiğin (0,5) altında kalır.
- **Çakışma:** en uzun aday kalır; eşitse en yüksek skorlu.
- **Konumlar** girişin code point'leri üzerindedir, altın setle aynı.

## Reddedilenler

- **`regex` crate'i (v0.1 için):**
  - Örtüşen eşleşme ve lookaround olmadığı için yan yana numaralar ve "önünde harf yok" koşulu yine kodla çözülür.
  - Ayraç kuralı her kalıpta tekrarlanır.
  - Sonuç yine iki katmanlı olur: regex aday bulur, kod doğrular.
- **Bağlamsız VKN'yi her zaman bulmak:** Sipariş ve müşteri numaraları da maskelenir.

## Sonuç

Bu karar regex'i reddetmez. v0.5'te kalıba dayalı tanıyıcılar gelecek: plaka, pasaport, IP, MAC, API anahtarı. O gün `regex` crate'i ve PRD'deki bildirimsel (TOML) tanıyıcı biçimi eklenir; iki yaklaşım yan yana yaşar. Yazıyla yazılmış numaralar da v0.5'te gelir.

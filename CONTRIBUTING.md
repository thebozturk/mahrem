# Katkı rehberi

Katkın için teşekkürler. Başlamadan önce bu kuralları oku.

## Gerçek kişisel veri yok

Testlerde, örneklerde, issue'larda ve pull request'lerde gerçek kişisel veri kullanılmaz. Yalnızca sentetik değerler kabul edilir:

- Kimlik numaraları için desen numaraları (`11111111110` gibi) ya da çekirdekteki üreticiler.
- Kart numaraları için kart ağlarının test numaraları.
- Adlar ve adresler için uydurma değerler.

Gerçek veri içeren bir katkı, geçmişten temizlenerek reddedilir.

## Bağımlılıklar

- Yalnızca izinli lisanslar: MIT, Apache-2.0, BSD, ISC, Zlib, Unicode-3.0. `cargo deny check` CI'da çalışır.
- Rust bağımlılıkları yalnızca crates.io'dan gelir; git bağımlılığı eklenmez.
- `unsafe` kod yasaktır. Tek istisna PyO3 bağlaması (`mahrem-py`).
- GitHub Action'lar commit SHA'sına sabitlenir.

## Terim

Geri çevrilebilen maskeleme takma adlandırmadır. Kodda, belgelerde ve çıktılarda Mahrem "anonimleştirme aracı" olarak anılmaz.

## Akış

1. Bir issue aç ya da var olan bir issue'yu üstlen. Büyük değişiklikleri önce issue'da konuşalım.
2. `make check` ve gerekiyorsa `make py-test` yerelde geçmeli.
3. Commit'ler DCO ile imzalanır: `git commit -s`. Bu, katkının sana ait olduğunu ve Apache 2.0 ile paylaşılabileceğini beyan eder ([developercertificate.org](https://developercertificate.org/)).
4. Yeni bir doğrulayıcı ya da dönüşüm eklendiğinde conformance setine de vaka eklenir.

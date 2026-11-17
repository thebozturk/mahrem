# 0004: Etiketler çekirdekte, kayıt tipleri `mahrem-dataset`'te

Durum: kabul edildi
Tarih: 2026-11-18

## Bağlam

Altın set şeması için iki parça gerekiyor: etiket listesi ve JSONL kayıt tipleriyle doğrulayıcı. "Tek motor" sınırı tespit ve maskeleme mantığının `mahrem-core`'da yaşamasını, çekirdeğin çalışma zamanı bağımlılığı olmamasını istiyor.

## Karar

- **`Label` ve `CodePointIndex` `mahrem-core`'dadır.** Etiket motorun kelime dağarcığıdır: `analyze()` bu türleri döndürecek, placeholder adını bu belirleyecek. Bench ya da Python tanımı yeniden yazmaz.
- **Kayıt tipleri ve doğrulayıcı yeni bir crate'tedir: `mahrem-dataset`.** Bağımlılıkları `mahrem-core`, `serde`, `serde_json`, `unicode-normalization`. `publish = false`.
- **Python bağlaması şimdilik yok.** İlk Python kullanıcısı geldiğinde `mahrem-py` üzerinden açılır, Python'da yeniden yazılmaz.

## Reddedilenler

- **Her şeyi `mahrem-core`'a koymak:** Çekirdeğe serde ve Unicode normalleştirme bağımlılığı girer. Kayıt formatı motorun değil, veri araçlarının işidir.
- **`mahrem-bench` içine koymak:** Harness, veri üretimi ve etiketleme hattı da aynı tipleri kullanacak. Bir binary crate'e bağımlı olamazlar.
- **Doğrulayıcıyı Python'da yazmak:** İki uygulama zamanla ayrışır; tek motor ilkesine aykırıdır.

## Sonuç

Bağımlılık yönü `mahrem-dataset → mahrem-core`. Bench, harness ve veri üretimi aynı kayıt tiplerini kullanır. Çekirdek bağımlılıksız kalır.

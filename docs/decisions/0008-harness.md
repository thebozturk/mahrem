# 0008: Harness LLM'den bağımsızdır; bitmiş örneklerin kaynağı shard'lardır

Durum: kabul edildi
Tarih: 2026-11-18

## Bağlam

Veri üretimi ve ölçüm saatler süren toplu işlerdir. İş her an kesilebilir (Ctrl-C, çökme, elektrik); kaldığı yerden sürmeli, ne örnek kaybetmeli ne de iki kez yazmalı. Her çıktının kökeni (plan, model, tohum, kod sürümü) bilinmeli.

## Karar

- **`mahrem-harness` `mahrem-llm`'e bağımlı değildir.** Harness yalnızca planı, aşamayı (`Stage`) ve durumu bilir. LLM'i aşamayı yazan üst binary (`mahrem-datagen`, `mahrem-bench`) çağırır. Bağımlılık yönü: `datagen → harness, llm, dataset, core`.
- **Plan deterministiktir.** TOML plandaki tohumdan her örneğin kimliği ve tohumu türer: `SHA-256(tohum + "\n" + sıra)`. Planın özeti biçimden bağımsızdır. Bir çıktı dizini başka bir planla ya da başka bir kökenle (model, ağırlık özeti) sürdürülemez.
- **Tek yazıcı, önce yazım.** Kabul edilen çıktı JSONL shard'ına tek yazımla eklenir ve diske indirilir (`fsync`); ancak ondan sonra SQLite'ta işaretlenir.
- **Bitmiş örneklerin doğruluk kaynağı shard'lardır.** Açılışta son shard'daki yarım satır kesilir. Shard'da olup veritabanında olmayan örnek bitmiş sayılır; veritabanında olup shard'da olmayan yeniden çalıştırılır. Veritabanı reddedilenleri, başarısızları ve deneme sayılarını tutar.
- **Üç hata sınıfı:** geçici (yeniden denenir; tükenirse sonraki çalıştırmada yine denenir), ret (gerekçesi kaydedilir, yeniden çalıştırılmaz), ölümcül (yeni örnek başlamaz, çalıştırma hatayla döner).
- **Durdurmada süren örnekler bitirilir**, yenisi başlatılmaz.
- **SSE ve yanıt önbelleği ertelendi.** SSE gateway'le, önbellek `mahrem-eval`'le gelir; veri üretimi ikisini de kullanmaz.

## Reddedilenler

- **Harness'ın LLM'i doğrudan çağırması:** LLM'siz işler (şablonla üretim, ölçüm) için gereksiz bağımlılık; testler sahte sunucu ister.
- **Durumu yalnızca SQLite'ta tutmak:** Satır yazılıp işaret düşmeden çökerse örnek ya kaybolur ya iki kez yazılır.
- **`async-trait` ve `anyhow`:** Yerleşik trait async fonksiyonları ve kutulanmış hata yetiyor; iki bağımlılık daha az.

## Sonuç

Süreç rastgele anlarda SIGKILL ile öldürülerek test edilir; kayıp ya da çift örnek çıkmaz. Her örnekte bir `fsync` var; yerel LLM'in hızında bu fark edilmez. SQLite için `rusqlite` 0.41 (gömülü SQLite 3.54) seçildi. Daha eski sürümlerin gömdüğü SQLite'ta güvenlik açığı var. Bu yüzden workspace'in en düşük Rust sürümü 1.89'a çıktı.

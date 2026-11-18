# 0010: Sahte değer üreticileri çekirdekte, listeler bu projede yazıldı

Durum: kabul edildi
Tarih: 2026-11-18

## Bağlam

Sentetik veri hattında LLM metni yer tutucularla yazar, değerleri bizim üreticilerimiz yerleştirir. Bu yüzden etiketler kesin doğrudur. v0.5'teki "Türkçe sahte veri" operatörü de aynı değerlere ihtiyaç duyar. PRD değerler için Faker tr_TR'yi ve kendi checksum'lı üreticilerimizi önerdi.

## Karar

- **Üreticiler `mahrem-core::fake` modülündedir.** Veri hattı ve sahte veri operatörü aynı kodu kullanır.
- **Çekirdek bağımlılıksız kalır.** Tohumlu rastgele sayı üreteci (SplitMix64) modülün içinde yazılı. Aynı tohum her sürümde ve her platformda aynı değeri verir.
- **Ad, soyad, il, ilçe, cadde ve kurum listeleri bu projede yazıldı.** Adlar ve yer adları olgudur. Siyasi parti, dernek, sendika ve banka adları uydurmadır.
- **Kimlik ve finans numaraları checksum'ı doğru üretilir.** Her etiketin değeri altın set doğrulayıcısından uyarısız geçer; bu testte denetlenir.
- **Üretici yalnızca değeri döner.** Hitap, alan adı ve yüklem çevredeki metne aittir; span sınırları etiketleme yönergesine uyar.

## Reddedilenler

- **Faker tr_TR:** Python'dadır; Rust çekirdeğine ya bağımlılık ya da veri kopyası olarak girerdi. Checksum'lı numaraları zaten kendimiz üretmemiz gerekiyordu.
- **`rand` crate'i:** Çıktısı sürümler arasında değişebilir; tohumla tekrarlanabilirlik bozulur.
- **Ayrı bir crate:** Tek kullanıcısı çekirdeğin kendi operatörü olacak; ayrı crate yalnızca sınır ekler.

## Sonuç

Değerler rastgeledir. Checksum'ı doğru bir numaranın gerçek bir kişiye denk gelmesi tesadüftür ve değer başka hiçbir bilgiyle ilişkilendirilmez. Listeler küçüktür; çeşitlilik gerekirse listeler büyütülür.

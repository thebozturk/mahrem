# Güvenlik politikası

Mahrem bir güvenlik aracıdır; açıkları ciddiye alıyoruz.

## Açık bildirimi

Güvenlik açıklarını herkese açık issue olarak açma. GitHub'ın **Private vulnerability reporting** özelliğini kullan: repo sayfasında Security → Report a vulnerability.

Bildirime şunları ekle:

- Etkilenen sürüm ya da commit.
- Açığı yeniden üretme adımları (yalnızca sentetik veriyle).
- Olası etki: ör. maskelenmemiş verinin dışarı çıkması, vault'a yetkisiz erişim.

## Kapsam

Özellikle şunlar güvenlik açığı sayılır:

- Maskeleme hata verdiğinde ham metnin LLM'e gitmesi (fail-closed ihlali).
- Etiket enjeksiyonuyla başka bir kaynağın gerçek değerlerinin görülmesi.
- Loglarda, metriklerde ya da denetim kaydında ham değer bulunması.

Tespit kaçırma (recall eksikliği) güvenlik açığı değil, kalite sorunudur; normal issue olarak bildirilebilir.

## Desteklenen sürümler

Proje henüz ilk sürümünü yayınlamadı. Yayından sonra yalnızca son sürüm desteklenir.

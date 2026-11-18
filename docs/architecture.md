# Mimari

Bu belge crate sınırlarını ve bağımlılık yönünü anlatır. Ürünün ne yaptığı ve veri akışı (tespit → maske → vault → geri yerleştirme) [README.md](../README.md)'deki "How it works" bölümündedir; burada tekrar edilmez.

## Bugünkü durum

```mermaid
flowchart TD
    Py["mahrem-py<br/>PyO3 bağlaması"] --> Core["mahrem-core<br/>doğrulayıcılar, harf dönüşümü,<br/>etiketler, konum dönüşümü"]
    Dataset["mahrem-dataset<br/>altın set kayıtları, doğrulayıcı"] --> Core
    Harness["mahrem-harness<br/>plan, aşama, shard, durum"]
    Llm["mahrem-llm<br/>OpenAI uyumlu istemci"]
    Conf[("conformance/<br/>ortak vakalar")] -. "okur" .- CoreTest["mahrem-core testleri"]
    Conf -. "okur" .- PyTest["mahrem-py testleri"]
```

- Kod `mahrem-core`, `mahrem-py`, `mahrem-dataset`, `mahrem-harness` ve `mahrem-llm` içindedir. Diğer crate'ler yer tutucudur ve hiçbir bağımlılıkları yoktur.
- `mahrem-core`'un çalışma zamanı bağımlılığı yoktur. Etiket listesi (`Label`) burada durur, çünkü motorun çıktısı bu türlerle ifade edilir.
- `mahrem-py` yalnızca çekirdeği dışarı açar; kendi mantığı yoktur.
- `mahrem-dataset` motor değil, veri aracıdır: altın set ve tahmin dosyalarının kayıt tipleri, doğrulayıcısı, dağılım raporu, açık/kapalı bölmesi ve kapalı bölüm özeti ([format](gold-set/README.md), [kompozisyon](gold-set/composition.md)). Bench ve veri üretimi bunu kullanacak. Gerekçe: [0004](decisions/0004-veri-seti-crate.md).
- `mahrem-harness` toplu iş çalıştırıcıdır: deterministik plan, kaldığı yerden devam, köken kaydı. Hiçbir Mahrem crate'ine bağımlı değildir; LLM'i bilmez. Aşamayı yazan üst binary LLM'i çağırır. Gerekçe: [0008](decisions/0008-harness.md).
- `mahrem-llm` OpenAI uyumlu istemcidir: yeniden deneme, eşzamanlılık sınırı, token sayacı. Hiçbir Mahrem crate'ine bağımlı değildir. Varsayılan olarak yalnızca yerel uca bağlanır: [0009](decisions/0009-llm-yerel-uc.md).

## Değişmeyecek sınırlar

- **Tek motor.** Tespit, maskeleme ve geri yerleştirme mantığı `mahrem-core`'da yaşar. Python paketi, gateway ve benchmark bu mantığı yeniden yazmaz, çağırır.
- **Conformance bunu doğrular.** Aynı girdi her yüzeyde aynı çıktıyı vermelidir; vakalar `conformance/` altındadır.
- **`unsafe` yalnızca `mahrem-py`'de.** Workspace'in geri kalanında `unsafe_code = "forbid"`.

## Planlanan, henüz yazılmamış

Crate başlıklarında yazan niyetlerdir; kod gelince bu bölüm "Bugünkü durum"a taşınır.

- `mahrem-datagen` (veri üretimi) `mahrem-harness`, `mahrem-llm` ve `mahrem-dataset`'i birleştirecek. `mahrem-llm`'i ayrıca `mahrem-gateway` kullanacak.
- `mahrem-tools`'u ajanlar ve MCP sunucusu kullanacak.
- `mahrem-ner` modeli ONNX Runtime ile CPU'da çalıştıracak.
- `mahrem-vault` etiket ile gerçek değer eşlemesini şifreli saklayacak.

Bir crate ilk kez doldurulurken sınırı ve bağımlılıkları burada güncellenir; gerekçesi [decisions/](decisions/README.md) altına yazılır.

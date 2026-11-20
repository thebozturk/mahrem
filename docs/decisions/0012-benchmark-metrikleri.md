# 0012: Benchmark metrikleri: sızıntı etiketten bağımsız, payda kişisel veri içeren belgeler

Durum: kabul edildi
Tarih: 2026-11-20

## Bağlam

Ana metrik sızıntısız belge oranıdır. "Kaçmadı"nın ne demek olduğu, oranın paydası ve v0.1'de yalnızca yapısal veri bulan bir aracın nasıl ölçüleceği tanımlanmamıştı. Tasarım ortakları benchmark'ı kendi verileriyle çalıştırıp yalnızca sayıları paylaşacak.

## Karar

- **Sızıntı etiketten bağımsızdır.** Bir altın span'in boşluk dışındaki her karakteri herhangi bir tahminle kaplıysa korunmuş sayılır. Yanlış etiketle maskelenen veri LLM'e gitmez. Yarısı maskelenmiş değer kaçmış sayılır. Etiket doğruluğu kesin ve örtüşmeli recall/precision ile ayrıca ölçülür.
- **Sızıntısız belge oranının paydası** en az bir kişisel veri içeren belgelerdir. Kişisel veri içermeyen belgeler yanlış alarm oranında ölçülür.
- **Kapsam (`--scope`):** `all`, `structural`, `high-risk`. Kapsam dışı etiketli tahminler maskeledikleri karakterleri korur, ama precision ve yanlış alarm hesabına girmez. Kapı 1 yapısal kapsamla ölçülür.
- **Rapor yalnızca sayı içerir.** Metin, değer ve tam dosya yolu yazılmaz. Kaçanların listesi bile yalnızca belge kimliği, konum ve etiket gösterir.
- **Metrik hesabı `mahrem-dataset`'tedir** (`eval`). `mahrem-bench` ince bir binary'dir; ileride Python'dan da çağrılabilir.

## Reddedilenler

- **Etiketin de doğru olmasını şart koşmak:** Gizlilik açısından fazla katı. Telefon numarasını `TCKN` diye maskeleyen araç veriyi sızdırmamıştır.
- **Paydaya bütün belgeleri koymak:** Kişisel veri içermeyen belgeler oranı yapay olarak yükseltir.
- **Her zaman bütün etiketlerle ölçmek:** Yalnızca yapısal veri bulan bir araçta her belge sızıntılı görünür; sayı bir şey söylemez.

## Sonuç

Raporlar tasarım ortağının ortamından güvenle çıkabilir. Bir aracın etiketleri yanlış ama maskelemesi tam olabilir; bu durum korunma oranı yüksek, kesin recall düşük olarak görünür.

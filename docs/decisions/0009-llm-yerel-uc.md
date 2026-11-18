# 0009: LLM istemcisi varsayılan olarak yalnızca yerel uca bağlanır

Durum: kabul edildi
Tarih: 2026-11-18

## Bağlam

Maskelenmemiş veriyle çalışan her LLM yerel ya da kurum içi olmalı. Altın setin kapalı bölümü hiçbir dış servise gidemez (karar [0007](0007-altin-set-kompozisyonu.md)). Bu kural yalnızca belgede kalırsa, yanlış bir adres ya da ortam değişkeni veriyi dışarı gönderebilir.

## Karar

- `mahrem-llm` varsayılan olarak yalnızca `localhost`, 127.0.0.0/8 ve `::1` adreslerine bağlanır. Başka bir adres `allow_remote` ayarını açıkça ister; bu ayar çalıştırmanın köken kaydına yazılır.
- Sistem vekil sunucusu (`HTTP_PROXY` …) kullanılmaz, yönlendirmeler izlenmez. İstek yalnızca denetlenen adrese gider.
- Şimdilik yalnızca `http` desteklenir. TLS, ilk uzak uç ihtiyacıyla (gateway) gelir. `reqwest`'in TLS kökleri izin listesinde olmayan bir lisans (CDLA-Permissive-2.0) getiriyor; o gün ayrıca karar verilir.

## Reddedilenler

- **Kuralı yalnızca belgelemek:** Kapalı bölümün sızması geri alınamaz; koruma kodda olmalı.
- **Özel ağ adreslerini (10/8, 192.168/16) yerel saymak:** Kurum içi olduğu adresten anlaşılmaz. Bu yüzden açık bir karar ister.

## Sonuç

Kurum içi bir sunucuya bağlanmak bir ayar gerektirir ve bu ayar kayda geçer. Gateway maskelenmiş veriyi dış sağlayıcılara gönderecek; o istemci `allow_remote` ile kurulur.

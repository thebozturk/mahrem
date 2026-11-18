# Durum

Bu dosya "nerede kaldık" sorusunun cevabıdır. Kısa kalır ve her iş bitiminde üzerine yazılır; geçmiş `git log`'dadır, büyük plan [ROADMAP.md](ROADMAP.md)'dedir.

Son güncelleme: 2026-11-18

## Aşama

**Hazırlık** (ölçüm ve veri). Çıkış kapısı: model kararı.

## Son biten

- Zor örnekler seti: [datasets/hard-cases/hard-cases.jsonl](datasets/hard-cases/hard-cases.jsonl). 82 belge, 134 span, 11 belgede kişisel veri yok; her etiket ve her zor durum türü en az bir kez geçiyor. CI'da hatasız ve uyarısız doğrulanıyor
- Açık veri setleri `datasets/` altında, CC BY 4.0. Karar: [0006](docs/decisions/0006-acik-veri-setleri.md)
- Doğrulayıcı: alan koduyla başlayan telefon numarası (`(0212) …`) artık uyarı vermiyor
- Etiketleme yönergesi 0.2.0: mesajın alıcısı olan kurum etiketlenmez
- Daha önce: etiketleme yönergesi 0.1.0, `ACCOUNT_ID`, altın set şeması

## Sıradaki

1. Altın set v0: açık ve kapalı bölüm. İçerik ve üretim yolu (elle, şablon, yerel LLM) kararlaştırılmalı
2. Rust harness ve LLM istemcisi (`mahrem-harness`, `mahrem-llm`)
3. Etiketleme iş akışı: tanıyıcılar, NER ve sözlükler; uyuşmazlıkta yerel LLM; Argilla

## Yarım kalan iş

Yok.

## Açık sorular

- Doğrulayıcının komut satırı arayüzü yok; `mahrem-bench` gelince eklenir.

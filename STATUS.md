# Durum

Bu dosya "nerede kaldık" sorusunun cevabıdır. Kısa kalır ve her iş bitiminde üzerine yazılır; geçmiş `git log`'dadır, büyük plan [ROADMAP.md](ROADMAP.md)'dedir.

Son güncelleme: 2026-11-18

## Aşama

**Hazırlık** (ölçüm ve veri). Çıkış kapısı: model kararı.

## Son biten

- Etiketleme yönergesi 0.1.0: [docs/gold-set/labeling-guideline.md](docs/gold-set/labeling-guideline.md). PRD'deki kurallar, etiket bazında kapsam, KVKK m.6 span kuralı, çocuk işareti, belirsizlik ve sürümleme
- Yeni etiket `ACCOUNT_ID` (müşteri, abone, poliçe, hasta numarası); etiket sayısı 32. Karar: [0005](docs/decisions/0005-account-id-etiketi.md)
- Doğrulayıcı yönergeyle hizalandı: yakınlık sözcükleri (Teyze, Hoca…) hitap listesinde, değeri çevreleyen noktalama için uyarı
- Altın set şeması (schema_version 1): [docs/gold-set/](docs/gold-set/README.md), `mahrem-dataset`, kararlar 0002–0004

## Sıradaki

1. Zor örnekler seti: yönergedeki her kuralın en az bir doğrulanmış örneği ve PRD'deki zor durumlar. Bitince ROADMAP'teki "Etiketleme yönergesi ve zor örnekler seti" kutusu işaretlenir
2. Altın set v0: açık ve kapalı bölüm
3. Rust harness ve LLM istemcisi (`mahrem-harness`, `mahrem-llm`)

## Yarım kalan iş

Yok.

## Açık sorular

- Doğrulayıcının komut satırı arayüzü yok; `mahrem-bench` gelince eklenir.

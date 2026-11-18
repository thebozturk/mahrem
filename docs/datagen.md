# Veri üretimi

`mahrem-datagen` Türkçe sentetik metin üretir.
- Yerel bir LLM metni yer tutucularla yazar (`{KİŞİ_1}`, `{TELEFON_1}`).
- Değerleri çekirdeğin üreticileri yerleştirir ([0010](decisions/0010-sahte-deger-ureticileri.md)).
- Değeri araç koyduğu için span'lerin yeri ve etiketi kesindir.

## Akış

1. **Plan:** Her örneğin metin türü, senaryosu, varlıkları, çocuk işaretleri ve zor durumu tohumdan belirlenir. Oranlar [kompozisyon hedeflerini](gold-set/composition-v0.json) tutturacak ağırlıktadır.
2. **Prompt:** [`prompts/system.txt`](../crates/mahrem-datagen/prompts/system.txt) ve [`prompts/scenarios.toml`](../crates/mahrem-datagen/prompts/scenarios.toml). Senaryo bölme grubudur. Prompt'lar binary'ye gömülüdür; özetleri köken kaydına yazılır.
3. **Denetim:** Eksik, fazla ya da bozuk yer tutucu ve yer tutucu dışında kalmış veri (numara, e-posta, listedeki adlar) aranır. Bulunan örnek gerekçesiyle reddedilir.
4. **Doldurma:** Yer tutucuya eklenen eke uyan değer seçilir ("Ahmet'in", "Ayşe'nin"). Uyan değer yoksa örnek reddedilir.
5. **Zor durum:** Türkçe karaktersiz yazım, büyük ya da küçük harf, yazım hatası, boşluklu, satıra bölünmüş ya da yazıyla yazılmış numara, "at/nokta"lı e-posta. Belirsiz ad ve kişi adlı yer, doldurmada seçilir.
6. **Doğrulama:** Kayıt altın set doğrulayıcısından hiçbir bulgu olmadan geçmelidir.
7. **Birleştirme (`build`):** Tekilleştirme, grup düzeyinde bölme, açık ve kapalı dosya, iki bölüm için kompozisyon raporu ve kapalı bölüm özeti.

## Gereksinimler

- **Yerel, OpenAI uyumlu bir `chat/completions` sunucusu:** örneğin `mlx_lm.server`, `llama.cpp` sunucusu ya da vLLM.
- **İstemci varsayılan olarak yalnızca yerel adreslere bağlanır** ([0009](decisions/0009-llm-yerel-uc.md)). Kurum içi bir sunucu için `allow_remote = true` gerekir; bu ayar köken kaydına yazılır.
- **Modelin lisansı:** Kullandığın modelin lisansı, çıktısıyla model eğitmene izin vermeli.

Derleme:

```sh
cargo build --release -p mahrem-datagen
```

## Plan

```toml
name = "deneme"            # örnek kimliklerinin öneki; çalıştırmalar arasında farklı olmalı
seed = "deneme-1"
items = 50
concurrency = 1            # sunucunun aynı anda işleyebildiği istek sayısı
max_attempts = 3           # geçici hatalarda (zaman aşımı, sunucu hatası)

[job]
endpoint = "http://127.0.0.1:8080/v1"
model = "<sunucunun bildiği model adı>"
model_revision = "<ağırlıkların sürümü, örneğin Hugging Face commit SHA'sı>"
temperature = 0.9
max_tokens = 700
# top_p = 0.95
# timeout_secs = 300
# allow_remote = false
```

```sh
mahrem-datagen run deneme.toml --out runs/deneme
```

- Kesilirse (Ctrl-C, çökme) aynı komut kaldığı yerden sürer.
- Aynı çıktı dizini başka bir planla ya da başka bir modelle sürdürülemez.
- Komut sonunda kabul, ret ve ret gerekçeleri yazılır.
- İlerleme logları saniyedeki token sayısını gösterir.

## Elle yazılmış belgeler

Altın set kompozisyonu her bölümde elle yazılmış belgeler ister. Metinler yer tutucuyla yazılır, satır başına bir JSON nesnesi olarak:

```json
{"id": "hw-001", "domain": "chat", "text": "Kızım {KİŞİ_1}'in {SAĞLIK_1}'ı var, {TELEFON_1}'dan ulaşabilirsiniz.", "child": ["KİŞİ_1"], "values": {"SAĞLIK_1": "astım"}}
```

- **Yer tutucu adları:** etiketlerin yer tutucu adları ve sıra numarası (`KİŞİ_1`, `E_POSTA_2`). Liste [gold-set/README.md](gold-set/README.md)'dedir.
- **`child`:** çocuğa ait kişi ve yaş yer tutucuları.
- **`values`:** sabit değerler. Özel nitelikli veride anlamı metin belirlediği için verilmesi önerilir.
- **`hard_cases` ve `group`:** isteğe bağlı.

```sh
mahrem-datagen fill elle.jsonl --seed 1 --out elle.filled.jsonl
```

## Kör okuma

Birkaç çalıştırmanın metinleri, hangi modelden geldiği gösterilmeden karıştırılır. Modelleri Türkçe doğallığına göre karşılaştırmak için:

```sh
mahrem-datagen blind --run runs/a --run runs/b --per-run 25 --out kor-okuma
```

`kor-okuma/metinler.md` okunur ve puanlanır. Hangi metnin hangi modelden geldiği `anahtar.json`'dadır.

## Setin kurulması

```sh
mahrem-datagen build --run runs/gold-v0 --handwritten elle.filled.jsonl \
  --against datasets/hard-cases/hard-cases.jsonl --seed <sürüm tohumu> --out gold-v0
```

Çıktılar:
- `gold-v0.open.jsonl` ve `gold-v0.closed.jsonl`;
- `gold-v0.report.md`: iki bölümün dağılımı, hedeften sapmalar ve kapalı bölüm özeti.

Sapma varsa planın örnek sayısı artırılmaz, çünkü plan değişirse çalıştırma sürdürülemez. Bunun yerine başka adla ikinci bir plan çalıştırılır ve `build`'e iki `--run` verilir.

Kapalı bölüm hiçbir dış servise gönderilmez ve bulutla eşitlenen bir dizinde tutulmaz ([kompozisyon](gold-set/composition.md#closed-part)).

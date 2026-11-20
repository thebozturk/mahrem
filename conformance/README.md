# Conformance seti

Aynı girdi Rust çekirdeği, Python paketi ve gateway üzerinden aynı çıktıyı vermelidir. Bu klasördeki dosyalar her dilin testleri tarafından okunur.

| Dosya | İçerik |
| --- | --- |
| `validators.jsonl` | Doğrulayıcılar ve Türkçe harf dönüşümü: `{"func", "input", "expected"}` |
| `analyze.jsonl` | Yapısal tanıyıcılar, varsayılan eşikle: `{"text", "expected": [{"label", "start", "end", "text", "source"}]}`. Konumlar code point cinsinden; skor karşılaştırılmaz |

Kurallar:

- Yalnızca sentetik değerler kullanılır. Kimlik numaraları desen numaralarıdır (`11111111110` gibi), kart numaraları kart ağlarının test numaralarıdır.
- Yeni bir fonksiyon eklendiğinde hem `crates/mahrem-core/tests/conformance.rs` hem `crates/mahrem-py/tests/test_conformance.py` güncellenir.

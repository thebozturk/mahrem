# 0003: Etiket kimlikleri, KVKK m.6 gruplaması ve çocuk verisi

Durum: kabul edildi
Tarih: 2026-11-18

## Bağlam

Altın set, benchmark ve çekirdeğin `analyze()` çıktısı aynı etiket listesini kullanacak. Etiketin veri dosyasındaki adı ile LLM'e giden placeholder adı (`<KİŞİ_1>`) farklı ihtiyaçlara cevap veriyor. KVKK m.6/1 on üç özel nitelikli veri türü sayıyor. Çocuk verisi PRD'de ayrı bir kategori olarak geçiyor.

## Karar

- **Kimlik ile placeholder adı ayrıdır.** Kimlik kalıcı, ASCII, İngilizce, `BÜYÜK_HARF` (`PERSON`). Türkiye'ye özgü numaralar kendi kısaltmasını korur (`TCKN`, `VKN`, `YKN`, `MERSIS`). Placeholder adı Türkçe varsayılandır (`KİŞİ`) ve politikayla değişebilir. Kimlik hiçbir zaman değiştirilmez ya da yeniden kullanılmaz.
- **KVKK m.6/1 on etikete iner.** Kanunun birlikte saydığı ve etiketleyicinin güvenilir biçimde ayıramayacağı maddeler birleşir: ırk ve etnik köken (`ETHNIC_ORIGIN`); felsefi inanç, din, mezhep ve diğer inançlar (`BELIEF`). Diğer maddelerin her biri ayrı etikettir.
- **Gizli bilgiler tek etikettir** (`SECRET`): API anahtarı, JWT, özel anahtar ve parola. Etiketleyici türü her zaman ayıramaz; tespit zaten türe özgü kurallarla yapılır.
- **Çocuk verisi etiket değil işarettir:** `PERSON` ve `AGE` span'lerinde `child: true`.
- **Yüksek riskli grup PRD'deki ölçüm tanımıdır:** TCKN, YKN, VKN, IBAN, kart ve KVKK m.6. `Label::is_high_risk()` içinde durur; değiştirmek yeni karar kaydı ister.

## Reddedilenler

- **Türkçe placeholder adını kimlik yapmak (`KİŞİ`):** ASCII değil, politikayla değişebilen bir adı kalıcı kimliğe bağlar.
- **Türkçe ASCII kimlik (`KISI`, `SAGLIK`):** Açık seti okuyacak Presidio, GLiNER ve HF kullanıcıları için okunması zor; Türkçe adlar zaten placeholder'da.
- **On üç ayrı m.6 etiketi:** Irk ile etnik köken, din ile mezhep arasındaki ayrım etiketleyici uyumunu düşürür. Maskeleme ve ölçüm açısından fark yaratmaz.
- **Ayrı `CHILD` etiketi:** Çocuğun adı yine bir kişi adıdır. Ayrı etiket, NER'e aynı yüzey biçimi için iki sınıf öğretir; "çocuk" bilgisi bağlamdan gelir ("kızım Elif").
- **`SECRET` ve çocuk verisini yüksek riskli gruba almak:** Kapı ölçümleri PRD'deki tanımla kıyaslanabilir kalsın diye şimdilik alınmadı.

## Sonuç

Veri dosyaları placeholder adındaki değişikliklerden etkilenmez. Çocuk recall'u işaret üzerinden ayrıca raporlanabilir; politika ileride çocuk verisi için ayrı operatör seçebilir. Birleştirilen m.6 maddeleri sonradan ayrılmak istenirse yeni etiket ve yeniden etiketleme gerekir.

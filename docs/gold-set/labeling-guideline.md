# Etiketleme yönergesi

Sürüm: **0.1.0** (taslak)

> *English:* This is the labeling guideline for the Mahrem gold set, written in Turkish for the annotators. Every gold record states the guideline version it follows in `meta.guideline_version`. The file format is described in [README.md](README.md).

Bu yönerge altın setteki metinlerde neyin, hangi etiketle ve hangi sınırlarla etiketleneceğini anlatır. Dosya formatı ve etiket listesi [README.md](README.md)'dedir; burada tekrar edilmez.

Örneklerde köşeli parantez span'i gösterir: `[Ayşe]'ye` yalnızca `Ayşe`'nin etiketlendiği anlamına gelir.

## 1. Temel ilkeler

1. **Gizlilik önce gelir.** Bir ifadenin kişisel veri olup olmadığından emin değilsen etiketle ve `uncertain: true` işaretle. Kaçan bir veri, fazladan etiketlenen bir sözcükten daha pahalıdır.
2. **Bağlam belirler.** Aynı sözcük bir belgede kişi adı, başka bir belgede sıradan sözcük olabilir ("Umut geldi" ve "umut var"). Her geçişi kendi bağlamında değerlendir.
3. **Her geçiş etiketlenir.** Aynı değer belgede üç kez geçiyorsa üçü de etiketlenir.
4. **Düz etiketleme.** Span'ler çakışmaz. İç içe varlıklarda yalnızca en dıştaki etiketlenir: `[Ahmet Yılmaz İlkokulu]` tek `ORGANIZATION`'dır, içindeki ad ayrıca etiketlenmez.
5. **Kök etiketlenir, ek dışarıda kalır.** Kesme işareti olsun olmasın: `[Ahmet]'in`, `[Ahmet]in`, `[Ankara]'da`.
6. **Değer etiketlenir, çevresindeki sözcük etiketlenmez.** Hitap, unvan, alan adı ("Tel:", "TCKN:") ve yüklem span'in dışındadır: `Tel: [0555 000 00 00]`, `[diyabet] hastasıyım`.
7. **Metin olduğu gibi etiketlenir.** Yazım hatası, Türkçe karaktersiz yazım ya da büyük harf, span'in içeriğini değiştirmez; span yazılan biçimi kapsar: `[ayse yilmaz]`.

## 2. Span sınırları

| Konu | Kural | Örnek |
| --- | --- | --- |
| Ekler | Ek dışarıda. Kesme işareti de dışarıda | `[Mehmet Kaya]'nın`, `[İzmir]'deki` |
| Hitap ve unvan | Dışarıda: Bay, Bayan, Bey, Hanım, Hanımefendi, Beyefendi, Sayın, Sn., Dr., Av., Prof., Doç., Uzm., Op. | `Sayın [Mehmet Kaya] Bey`, `Dr. [Elif Şahin]` |
| Yakınlık ve hitap sözcükleri | Dışarıda: abi, abla, teyze, amca, dayı, hala, yenge, hoca, öğretmen | `[Fatma] Teyze`, `[Ali] Hoca` |
| Ad ve soyad | Bitişik ad ve soyad tek span | `[Mehmet Ali Kaya]` |
| Ayrık ad ve soyad | Araya başka sözcük giriyorsa iki span | `[Kaya], [Mehmet]` |
| Noktalama | Değeri çevreleyen tırnak, parantez, virgül ve nokta dışarıda. Değerin kendi noktalaması içeride | `"[Ayşe]"`, `([0555 000 00 00])`, `[Örnek Gıda Ltd. Şti.]` |
| Boşluk | Span boşlukla başlamaz ve bitmez | |
| Numaralar | Numara, aradaki boşluk, tire, nokta ve parantezlerle tek span. Ülke kodu içeride, alan adı dışarıda | `Tel: [+90 (555) 000 00 00]` |
| Satıra bölünmüş numara | Satır sonu dahil tek span | |
| Yazıyla numara | Yazılan ifadenin tamamı tek span | `[sıfır beş yüz elli beş ...]` |

## 3. Etiketler

### Kimlik ve finans numaraları

Bu numaralar sahibine bakılmaksızın etiketlenir: bir şirketin VKN'si de, örnek gibi görünen bir numara da etiketlenir. Kural katmanı numaranın kime ait olduğunu bilemez; ölçüm de bilmez.

| Etiket | Kapsam | Kapsam dışı |
| --- | --- | --- |
| `TCKN` | 11 haneli T.C. kimlik numarası | |
| `YKN` | 99 ile başlayan yabancı kimlik numarası | |
| `VKN` | 10 haneli vergi kimlik numarası; kişi ya da şirket | |
| `IBAN` | Türkiye ve diğer ülke IBAN'ları | Yalnızca banka adı ya da şube kodu |
| `CARD_NUMBER` | Tam kart numarası | Maskeli ya da kısmi numara ("son dört hanesi 1111"); son kullanma tarihi |
| `PASSPORT`, `DRIVER_LICENSE`, `SGK_NUMBER`, `MERSIS` | Belge ya da sicil numarasının kendisi | "Pasaport no:" gibi alan adları |

Kartın güvenlik kodu (CVV) ve tek kullanımlık doğrulama kodları `SECRET`'tir.

### İletişim

- `PHONE`: cep, sabit ve yurt dışı numaralar. Kurumsal numaralar da etiketlenir (444'lü, 0850'li); hangisinin kişiye ait olduğu metinden anlaşılmaz.
- `EMAIL`: adresin tamamı, kurumsal adresler dahil. "at" ve "nokta" ile yazılmışsa ifadenin tamamı: `[ayse at example nokta com]`.

### Araç, cihaz ve tarih

- `LICENSE_PLATE`: plakanın tamamı, boşluklarla: `[34 ABC 123]`.
- `SERIAL_NUMBER`: cihaz seri numarası ve IMEI. Sipariş, kargo takip ve fatura numaraları etiketlenmez.
- `IP_ADDRESS`, `MAC_ADDRESS`: adresin tamamı. Yerel ağ adresleri (192.168.x.x) de etiketlenir.
- `DATE`: yalnızca bir kişinin kimliğini ya da özel bir olayını belirleyen tarihler: doğum, ölüm, evlilik, sağlıkla ilgili tarihler (ameliyat, yatış, taburcu). Tarih ifadesinin tamamı tek span: `[12 Mart 1990]`, `[12.03.1990]`, `[1990] doğumlu`. Sipariş, teslimat, randevu ve fatura tarihleri etiketlenmez.

### Gizli bilgiler

`SECRET`: API anahtarı, erişim belirteci (JWT dahil), özel anahtar, parola, PIN, CVV ve tek kullanımlık doğrulama kodu. Yalnızca değer etiketlenir: `parolam: [Ornek.1234]`. Çok satırlı özel anahtar, başlık ve bitiş satırlarıyla birlikte tek span'dir.

### Kişi

`PERSON`: gerçek ya da gerçek gibi sunulan kişilerin adı ve soyadı.

- Tek başına ad da, tek başına soyad da etiketlenir: `[Kaya] Bey aradı`.
- Kamuya mal olmuş kişiler de etiketlenir. Maskelenip maskelenmeyeceğine politika dosyasındaki izin listesi karar verir.
- Kişiyi tanımlayan kullanıcı adları `PERSON`'dır: `[@ayse_demir]`.
- Kurgusal karakterler ("Keloğlan") ve markalaşmış adlar etiketlenmez.
- Yakınlık sözcükleri tek başına etiketlenmez: "annem", "oğlum", "müdürüm".
- Hem ad hem sözcük olan sözcüklerde (Umut, Deniz, Bahar) bağlam karar verir; bağlam yetmiyorsa etiketle ve `uncertain: true` işaretle.

### Adres

`ADDRESS`: mahalle düzeyinde ya da daha ayrıntılı adres. Bitişik adres parçaları (mahalle, cadde, sokak, numara, daire, ilçe, il, posta kodu) tek span'dir: `[Atatürk Caddesi No:5 Çankaya/Ankara]`.

- Yalnızca il ya da ilçe adı etiketlenmez: "Ankara'da yaşıyorum".
- Kişi adı taşıyan cadde ve okul adları adresin ya da kurumun parçasıdır, ayrıca `PERSON` olarak etiketlenmez.

### Kurum

`ORGANIZATION`: metinde bir kişiyle ilişkilendirilen kurum: çalıştığı işyeri, okuduğu okul, tedavi gördüğü hastane, müşterisi olduğu banka. Kurumun yasal eki içeride: `[Örnek Gıda A.Ş.]`.

- Bir kişiyle ilişkilendirilmeyen kurumlar etiketlenmez: haber konusu şirket, mesajın yazıldığı şirketin kendisi.
- Dernek, vakıf ve sendika üyeliği `MEMBERSHIP`'tir (aşağıda).

### Yaş

`AGE`: yalnızca sayı ya da yazıyla sayı. Birim ve ek dışarıda: `[7] yaşında`, `[otuz beş] yaşındayım`, `[18] aylık`.

### KVKK m.6: özel nitelikli veri

Özel nitelikli veri ancak **belirli bir kişiye** (yazan ya da üçüncü bir kişi) bağlandığında etiketlenir. Genel ifadeler etiketlenmez: "grip salgını başladı", "bu partinin oyları arttı".

Span, bilgiyi açığa çıkaran **değer sözcüğüdür**: hastalığın, inancın, partinin, sendikanın ya da kıyafetin adı. Yüklem ve ek dışarıda kalır. Maskelendiğinde bilgi gizlenmeli, cümle okunur kalmalıdır: `[diyabet] hastasıyım` → `<SAĞLIK_1> hastasıyım`.

| Etiket | Kapsam | Örnek |
| --- | --- | --- |
| `HEALTH` | Hastalık, tanı, belirti, ilaç, tedavi, engellilik, gebelik, test sonucu; sıradan geçici şikayetler dahil | `kızımın [astım]ı var`, `[insülin] kullanıyorum`, `[hamile]yim` |
| `BELIEF` | Din, mezhep, felsefi inanç; inancı açığa çıkaran ibadet | `[Alevi]yim`, `[ateist]im`, `[oruç] tutuyorum` |
| `ETHNIC_ORIGIN` | Irk ve etnik köken. Uyruk ve vatandaşlık değildir; ayrım belirsizse `uncertain` | `[Çerkez] kökenliyim` |
| `POLITICAL_OPINION` | Siyasi görüş, parti tercihi, parti üyeliği | `[X Partisi]'ne oy verdim` |
| `MEMBERSHIP` | Dernek, vakıf ya da sendika üyeliği. Kurum adı varsa o, yoksa kurum türü | `[sendika] üyesiyim`, `[Örnek-İş] üyesiyim` |
| `ATTIRE` | Dini, kültürel ya da kimlik belirten kıyafet. Sıradan giyim değildir | `[başörtüsü] takıyorum` |
| `SEX_LIFE` | Cinsel yönelim ve cinsel hayat | `[eşcinsel]im` |
| `CRIMINAL_RECORD` | Ceza mahkûmiyeti, güvenlik tedbiri, hapis, adli kontrol, sabıka, ceza soruşturması ve davası. Hukuk davaları (boşanma, alacak) değildir | `[hapis] yattı`, `[sabıka] kaydım var` |
| `GENETIC` | Genetik test sonucu, taşıyıcılık, mutasyon | `[BRCA1 mutasyonu] taşıyorum` |
| `BIOMETRIC` | Biyometrik verinin kendisi ya da onu temsil eden değer (şablon kodu gibi). Biyometrik bir sistemden söz edilmesi değildir | |

Kalıtsal bir hastalığın adı `HEALTH`, onu gösteren genetik test sonucu `GENETIC`'tir.

## 4. Çocuk verisi

Bir `PERSON` ya da `AGE` değeri 18 yaşından küçük birine aitse `child: true` işaretlenir.

- "kızım", "oğlum", "bebeğim", "öğrencim" gibi sözcükler, aksi anlaşılmadıkça çocuk sayılır: `kızım [Elif]` → `child: true`.
- Yaş 18'in altındaysa yaş da kişi de çocuktur.
- Bağlam yetişkin olduğunu gösteriyorsa işaretlenmez: "oğlum [Can] 30 yaşında".
- Diğer etiketler (çocuğun okulu, sağlık bilgisi) işaret taşımaz; kendi etiketleriyle etiketlenir.

## 5. Belirsizlik ve hakemlik

1. Emin olmadığın span'i etiketle ve `uncertain: true` işaretle. Hangi etiketin doğru olduğundan emin değilsen en olası etiketi seç ve yine işaretle.
2. `uncertain` işaretli span'lere ikinci etiketleyici ya da hakem bakar ve işareti kaldırır ya da span'i siler.
3. Hakemin kararı yeni bir durumu çözüyorsa bu yönergeye kural ya da örnek olarak eklenir ve değişiklik kaydına yazılır.

## 6. Belge düzeyinde

- Kişisel veri içermeyen belge de sete girer; `spans` boş kalır.
- `meta.hard_cases`, belgede bulunan zor durumları listeler (Türkçe karaktersiz yazım, boşluklu numara…). Değerler [README.md](README.md)'deki tablodadır.
- Belge bu yönergenin hangi sürümüne göre etiketlendiyse `meta.guideline_version` o sürümü taşır.

## 7. Sürümleme

- **Yama** (0.1.x): yazım düzeltmesi, yeni örnek. Etiketler değişmez.
- **Ara** (0.x.0): yeni kural ya da kapsamı genişleyen kural. Eski etiketler geçerli kalır ama yeni durumları eksik etiketlemiş olabilir.
- **Ana** (x.0.0): eski etiketlerin değişmesini gerektiren kural. Etkilenen belgeler yeniden etiketlenir.

## Değişiklik kaydı

| Sürüm | Değişiklik |
| --- | --- |
| 0.1.0 | İlk sürüm. PRD'deki yönerge tablosunun kuralları ve şema 1'in etiketleri |

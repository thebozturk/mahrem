# 0001: Ajan bağlamı repodaki dosyalarda tutulur

Durum: kabul edildi
Tarih: 2026-11-17

## Bağlam

Proje ağırlıklı olarak yapay zekâ ajanlarıyla geliştirilecek. Her yeni oturum boş bağlamla başlar; ajan projenin ne olduğunu, neyin nerede durduğunu ve hangi adımda kalındığını her seferinde yeniden çıkarmak zorunda kalır.

## Karar

Bağlam repoda, her biri tek bir soruya cevap veren dosyalarda tutulur:

- `AGENTS.md`: harita, komutlar, kurallar. Asıl kılavuz budur.
- `CLAUDE.md`: yalnızca `AGENTS.md` ve `STATUS.md`'yi içeri alır; Claude Code bu sayede ikisini her oturumda kendiliğinden yükler.
- `STATUS.md`: şu anki durum ve sıradaki adımlar. Üzerine yazılır.
- `docs/architecture.md`: crate sınırları.
- `docs/decisions/`: bu kayıtlar.

İş bitince `STATUS.md`'yi güncellemek ajanın görevidir; kural `AGENTS.md`'de yazar.

## Reddedilenler

- **Her şeyi tek `CLAUDE.md`'ye yazmak:** yalnızca Claude Code okur, dosya hızla büyür ve her oturumun bağlamını doldurur.
- **Durumu `ROADMAP.md`'de tutmak:** yol haritası kapıları anlatır, "dün nerede kaldık" ayrıntısını taşımaz.
- **Oturum günlüğü tutmak:** sürekli büyür, eskiyen satırlar ajanı yanıltır. Geçmiş zaten `git log`'dadır.

## Sonuç

Yeni bir oturum iki kısa dosyayla yönünü bulur. Karşılığında `STATUS.md` her işte güncellenmek zorundadır; güncellenmezse yanlış yönlendirir.

# LawSynth

LawSynth, zaman serisi gözlemlerinden çalıştırılabilir matematiksel dünyalar
üretmek için yerel ve belirlenimci bir araç takımıdır. Rust çalışma alanı World
IR doğrulaması, ifade değerlendirmesi, CSV verisinden seyrek yasa keşfi,
`.lsworld` paketleme ve sürekli/ayrık simülasyon sağlar.

## Mevcut kapsam

Çekirdek; doğrulanmış değişken ve parametre tanımları, RK4 sürekli simülasyonu,
ayrık adımlama, müdahaleler, sayısal CSV alma, türev ve özellik çıkarma, seyrek
regresyon, sonuç puanlama ve bütünlük denetimli paket okuma/yazmayı uygular.
CLI ve maturin ile derlenen Python bağlaması bu çekirdeği kullanır.

Henüz uygulanmayan servis, eklenti, Studio, nedensel çıkarım, rejim ve
belirsizlik katmanları çalışıyormuş gibi sunulmaz. Gerçek desteklenen davranış
için çalıştırılabilir örnekler ve uyumluluk testleri esas kaynaktır.

## Doğrulama

```sh
cargo test --workspace
cargo run -p lawsynth-cli -- --help
```

Katkı ve doğrulama süreci için [CONTRIBUTING.md](CONTRIBUTING.md) belgesine
bakın. Lisans [Apache-2.0](LICENSE)'dır.

# GridSynth Product

GridSynth, CSV şebeke ölçümlerini açıklanabilir yük modeli, anomali listesi ve altı saatlik müdahale senaryosuna çeviren tarayıcı tabanlı analiz ürünüdür.

## Çalışan kapsam

- Ürün içindeki [beş duraklı operatör rotası](./index.html#operatorRoute): kaynak, eşleme, analiz kaydı, senaryo ve rapor
- CSV yükleme, başlık algılama ve kullanıcı kontrollü sütun eşleme
- MW/kW/W, kV/V ve °C/°F kaynak birimi dönüşümü
- Eksik/geçersiz satır ile düzensiz zaman aralığı raporu
- Sıcaklık katsayılı yük modeli ve R² hesabı
- Model artığına göre incelenecek saatler
- Talep, sıcaklık ve dağıtık üretim müdahaleleri
- Gözlem, temel tahmin ve senaryo tahminini aynı grafikte karşılaştırma
- Hesaplanan JSON karar raporu
- Mobil ve masaüstü uyumlu, bağımlılıksız arayüz

## Ana akış ve durumlar

1. [Kaynak alanında](./index.html#data-source) kendi ölçüm CSV dosyanı seç. GridSynth yerleşik örnek veri yüklemez.
2. GridSynth dosyayı okuyunca [eşleme panelini](./index.html#mappingPanel) açar. Zaman, yük, gerilim, sıcaklık, isteğe bağlı kesinti ve kaynak birimlerini doğrula.
3. [Analiz kaydında](./index.html#dataContract) veri aralığını, model sürümünü, gerçek sütun adlarını, dönüşümleri ve atlanan satırları incele.
4. [Senaryo kontrollerinden](./index.html#scenario) talep, sıcaklık veya dağıtık üretim varsayımını değiştir.
5. `Raporu indir` ile güncel analizi JSON olarak dışa aktar.

[Durum yönetimi](./app.js) CSV bekleniyor, dosya okunuyor, eşleme gerekiyor, geçersiz veya boş dosya, dosya okuma hatası, analiz hatası ve analiz hazır hallerini ayrı metinlerle gösterir. Açılışta metrikler, grafik, model ve bulgular boştur; senaryo ile export kontrolleri geçerli analiz oluşana kadar kapalı kalır. GridSynth sonraki bir dosya tamamlanana kadar son geçerli yüklenmiş CSV analizini korur. Okuma veya doğrulama hatası oluşursa durum alanı ekranda kalan gerçek kaynağın adını açıklar.

GridSynth CSV'yi tarayıcı sekmesinde işler ve dış servis çağırmaz. Bu yüzden ağ veya servis izni durumu yoktur. Tarayıcının dosyayı okuyamaması erişim sınırını oluşturur; arayüz kullanıcıdan dosyayı yerel diske kopyalayıp yeniden seçmesini ister. CSV sözdizimi ve boş dosya hataları ayrı doğrulama mesajları üretir.

## Veri sözleşmesi

```csv
timestamp,load_mw,voltage_kv,temperature_c,outage_minutes
```

Bu adlar varsayılan sözleşmedir; yüklenen dosyanın başlıkları farklıysa kullanıcı zaman, yük, gerilim, hava/sıcaklık ve isteğe bağlı kesinti sütunlarını ekrandan bağlar. Yük `MW`, `kW` veya `W`; gerilim `kV` veya `V`; sıcaklık `°C` veya `°F` olarak alınabilir ve modelin kanonik `MW`, `kV`, `°C` birimlerine çevrilir. Kesinti alanı seçilmez veya hücre boş bırakılırsa sıfır kabul edilir.

Geçerli olmayan satırlar analizi bütünüyle durdurmaz: satır numarası ve gerekçesi veri sözleşmesi panelinde gösterilir. Panel ayrıca veri tarih aralığını, kullanılan gerçek CSV başlıklarını, beklenen örnekleme aralığını, düzensiz geçiş sayısını, dönüşümleri ve `gridsynth-linear-temperature-v1` model sürümünü açıklar. Aynı kayıt indirilen JSON raporunda `dataset` alanına yazılır. Rapor `source.classification: "uploaded-csv"`, gerçek dosya adı, eşleme, kaynak birimleri, veri aralığı, model ve bulguları taşır.

## Açma

`index.html` dosyasını doğrudan tarayıcıda aç veya bu klasörü herhangi bir statik dosya sunucusuyla yayınla. Derleme adımı yoktur.

## LawSynth bağlantısı

GridSynth bağımsız çalışan ince bir tarayıcı istemcisidir. Mevcut doğrusal model aynı veri sözleşmesi üzerinden LawSynth `discover → explain → forecast → report` hattından dönen `.lsworld` ve rapor verisiyle değiştirilebilir.

## Tasarım yönü

Organic anchor kullanıldı. Kum yüzey, yosun metni, toprak vurgu, Fraunces başlık ve Epilogue gövde ile altyapı verisi sakin bir karar yüzeyine taşındı. Müdahale kontrolleri grafiği, riski ve denklemi birlikte değiştirir.

## Doğrulama sınırı

Bu turda build, test, lint, CI ve tarayıcı sunucusu çalıştırılmadı. Kalan doğrulama: gerçek tarayıcıda ilk açılış kilitleri, geçerli CSV, boş CSV, bozuk tırnak, dosya okuma reddi, eksik eşleme, iki geçerli satırın altındaki veri, önceki gerçek sonucun korunması, responsive rota ve indirilen rapor içeriği.

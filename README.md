# LanShare Temiz Mimari Yeniden Yapılandırma Kılavuzu

Bu kılavuz, LanShare projesindeki modüllerin (crate'lerin) Single Responsibility (Tek Sorumluluk) ve Dependency Inversion (Bağımlılığın Tersine Çevrilmesi) prensiplerine göre nasıl yeniden yapılandırılacağını açıklar.

## 1. Yeni Oluşturulacak Çekirdek Modüller

### `lanshare-domain` (YENİ)

* **Görevi:** Sistemin kalbidir. İş kurallarını, ana veri yapılarını ve adaptörlerin uyması gereken sözleşmeleri (Trait) tanımlar.
* **Bağımlılıkları:** Hiçbir şeye bağımlı olamaz (Ne `std::fs`, ne `std::net`, ne de diğer LanShare modülleri).

### `lanshare-app` (YENİ)

* **Görevi:** Kullanım senaryolarını (Use Cases) barındırır. "Dosya Gönder", "Dosya Al", "Cihazları Listele" gibi işlemlerin adımlarını yönetir.
* **Bağımlılıkları:** Sadece `lanshare-domain` modülünü bilir. Ağı veya diski bilmez, `lanshare-domain` içindeki Trait'leri kullanarak iş yapar.

---

## 2. Mevcut Modüllerin Teşhis ve Tedavisi

### 📦 `lanshare-core`

* **Teşhis:** Klasik bir "Spagetti Kod" merkezidir. `server.rs` içinde TCP dinleniyor, `storage.rs` içinde diske yazılıyor, `transaction.rs` içinde hash hesaplanıp JSON kaydediliyor. Sorumluluklar tamamen iç içe geçmiş durumda.
* **Tedavi:** Bu modül tamamen parçalanacak ve **yok edilecek**.
1. Dosya sistemi (FS) işlemleri yeni açılacak `lanshare-storage` modülüne taşınacak.
2. TCP soket işlemleri yeni açılacak `lanshare-network` modülüne taşınacak.
3. Geriye kalan "İş akışı yönetimi" mantığı `lanshare-app` modülüne taşınacak.



### 📦 `lanshare-proto`

* **Teşhis:** Ağ üzerinden gönderilecek veri paketlerinin (Header, Payload) formatını belirlemesi gerekirken, `file_message.rs` içerisinde `std::fs::File` kullanarak doğrudan diske erişiyor ve hash hesaplıyor. Bir ağ protokolü diski bilmemelidir.
* **Tedavi:** Dosya IO işlemleri (`File::open` vb.) tamamen silinecek. Sadece byte dizilerini (`&[u8]`) veya genel `Read/Write` trait'lerini kullanarak mesajları serialize (paketleme) ve deserialize (açma) eden saf bir kütüphaneye dönüştürülecek.

### 📦 `lanshare-api`

* **Teşhis:** API yanıtları (`PeerApi`, `ApiError`, `DiscoveryApi` traiti) için kullanılmış. Ancak bu yapılar aslında sistemin çekirdek nesneleri.
* **Tedavi:** Bu modül silinecek veya içeriği tamamen yeni oluşturacağımız `lanshare-domain` modülünün içerisine taşınacak.

### 📦 `lanshare-discovery`

* **Teşhis:** Ağdaki diğer LanShare cihazlarını bulmak için mDNS ve UDP kullanıyor. Mevcut yapısı bağımsız çalışmaya oldukça uygun, thread ve channel mantığı fena değil.
* **Tedavi:** Temel işleyişi korunacak. Ancak dışarıya veri sunarken doğrudan kendi struct'larını dönmek yerine, `lanshare-domain` içindeki `DiscoveryPort` trait'ini implement edecek (uygulayacak). Böylece ana uygulama, mDNS kullanıldığını bilmeyecek, sadece "bana cihazları ver" diyecek.

### 📦 `lanshare-ipc` (Inter-Process Communication)

* **Teşhis:** Yüksek ihtimalle arka planda çalışan daemon (`lanshare-rs`) ile terminaldeki kullanıcı arayüzü (`lanshare-cli`) arasındaki haberleşmeyi sağlıyor (Unix socket veya Local TCP üzerinden).
* **Tedavi:** Bir Adaptör katmanı olarak kalacak. `lanshare-cli`'den gelen komutları alacak, bunları ayrıştırıp `lanshare-app` (Kullanım Senaryoları) katmanındaki ilgili fonksiyonları tetikleyecek (Örn: `SendFileUseCase.execute()`).

### 📦 `lanshare-cli`

* **Teşhis:** Kullanıcının terminalden komut girdiği yer (`lanshare send <ip> <file>`).
* **Tedavi:** Hiçbir iş mantığı (dosya okuma, hash alma) içermemeli. Sadece kullanıcının komutlarını `lanshare-ipc` üzerinden arka plandaki servise iletmeli ve dönen sonuçları ekrana yazdırmalı (UI).

### 📦 `lanshare-rs` (Main Daemon / Binary)

* **Teşhis:** Projenin ana çalıştırılabilir (executable) dosyası.
* **Tedavi:** Temiz Mimari'deki adı **Composition Root (Birleştirme Kökü)** olacak.
1. Başladığında `lanshare-storage` (Disk), `lanshare-network` (Ağ) ve `lanshare-discovery` (Keşif) adaptörlerini ayağa kaldıracak.
2. Bunları alıp `lanshare-app` (İş Mantığı) içindeki yöneticilere (Manager/UseCase) enjekte edecek (Dependency Injection).
3. Son olarak `lanshare-ipc`'yi dinlemeye başlayarak CLI'dan emir bekleyecek.



### 📦 `lanshare-tests`

* **Teşhis:** Entegrasyon testlerinin bulunduğu yer.
* **Tedavi:** Yeni mimari oturduktan sonra, gerçek disk ve gerçek ağ kullanmadan, sahte (Mock) adaptörlerle `lanshare-app` katmanını test etmek için kullanılacak.

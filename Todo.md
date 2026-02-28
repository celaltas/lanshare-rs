### 🗺️ LanShare Yeniden Yapılandırma Yol Haritası (To-Do List)

#### Faz 1: Çekirdek ve Sınırların Belirlenmesi (Domain)

* [x] **1.1.** Çalışma alanında `lanshare-domain` adında yeni bir library crate oluştur.
* [x] **1.2.** `lanshare-api/src/lib.rs` içindeki `PeerApi` ve `ApiError` yapılarını `lanshare-domain/src/models.rs` içine taşı ve isimlerini `Peer`, `DomainError` olarak genelleştir.
* [x] **1.3.** Sistemin diğer temel veri yapılarını (`FileManifest`, dosya transferi için `Chunk` vb.) `models.rs` içine ekle.
* [x] **1.4.** `lanshare-domain/src/ports.rs` dosyasını oluştur ve içine sistemin arayüzlerini (`StoragePort`, `NetworkPort`, `DiscoveryPort`) yaz.
* [x] **1.5.** Eski ve artık gereksiz olan `lanshare-api` crate'ini tamamen sil.

#### Faz 2: Protokol Katmanının Temizlenmesi (Proto)

* [x] **2.1.** `lanshare-proto/src/file_message.rs` dosyasına gir ve içindeki `std::fs::File`, `std::path::Path` gibi disk bağımlılıklarını tamamen sil.
* [x] **2.2.** Mesaj okuma/yazma fonksiyonlarını (`send`, `receive`, `send_partial`), doğrudan `std::fs::File` almak yerine sadece genel `std::io::Read` ve `std::io::Write` trait'lerini (veya byte dizilerini) kabul edecek şekilde refactor et. Böylece proto sadece ağ paketi yapmayı bilecek.

#### Faz 3: İş Mantığının Kurulması (Application / Use Cases)

* [x] **3.1.** `lanshare-app` adında yeni bir library crate oluştur ve `lanshare-domain`'i buna bağla.
* [x] **3.2.** `lanshare-app/src/use_cases/receive_file.rs` oluştur. Burada TCP'den bağımsız olarak "gelen byte'ları StoragePort'a yaz" mantığını kur.
* [x] **3.3.** `lanshare-app/src/use_cases/send_file.rs` oluştur. Burada "dosyayı StoragePort'tan oku, NetworkPort'a ver" mantığını kur.

#### Faz 4: Depolama Adaptörünün Çıkarılması (Storage Adapter)

* [x] **4.1.** `lanshare-storage` adında yeni bir library crate oluştur. Bağımlılık olarak `lanshare-domain`'i ekle.
* [x] **4.2.** `lanshare-core` içindeki `storage.rs` ve `transaction.rs` dosyalarını buraya taşı.
* [x] **4.3.** Taşınan bu yapıların `lanshare-domain::StoragePort` trait'ini implement etmesini (uygulamasını) sağla. (İşte burada diske yazma işlemi gerçekleşecek).

#### Faz 5: Ağ Adaptörünün Çıkarılması (Network Adapter)

* [x] **5.1.** `lanshare-network` adında yeni bir library crate oluştur. Bağımlılık olarak `lanshare-domain`, `lanshare-app` ve `lanshare-proto`'yu ekle.
* [x] **5.2.** `lanshare-core/src/server.rs` içindeki TCP soket dinleme mantığını buraya taşı.
* [x] **5.3.** Bu katmanın `lanshare-domain::NetworkPort` trait'ini implement etmesini sağla ve soketten gelen istekleri doğrudan `lanshare-app`'teki ilgili UseCase'lere pasla.
* [x] **5.4.** İçini tamamen boşalttığımız `lanshare-core` crate'ini projeden sil.

#### Faz 6: Keşif Adaptörünün Uyarlanması (Discovery Adapter)

* [x] **6.1.** Mevcut `lanshare-discovery` crate'ine `lanshare-domain` bağımlılığını ekle.
* [x] **6.2.** `lanshare-discovery/src/lib.rs` (veya `discovery_manager.rs`) içindeki ana yapının, `lanshare-domain::DiscoveryPort` trait'ini implement etmesini sağla. (Artık ana sistem, mDNS çalıştığını bilmeyecek, sadece `get_peers()` diyecek).

#### Faz 7: Birleştirme ve Haberleşme (Composition Root & IPC)

* [x] **7.1.** `lanshare-rs/src/main.rs` (Ana Daemon) içine gir. Storage, Network ve Discovery adaptörlerini burada ayağa kaldır (`new()` ile).
* [x] **7.2.** Bu adaptörleri `lanshare-app` yöneticilerine (Dependency Injection ile) enjekte et.
* [x] **7.3.** `lanshare-ipc` üzerinden gelen CLI komutlarını alıp `lanshare-app` katmanına yönlendir.

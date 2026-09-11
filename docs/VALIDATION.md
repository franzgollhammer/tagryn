# Prüfprotokoll

Historischer Stand: 10. September 2026. Dieses Protokoll beschreibt den ursprünglichen lokalen Entwicklungsbuild, nicht die Abnahme eines späteren Release-Commits. Die genannten `artifacts/`-Dateien bleiben lokale Prüfmaterialien und sind nicht Teil des veröffentlichten Quellcodes. Aktuelle CI-Ergebnisse stehen unter [GitHub Actions](https://github.com/franzgollhammer/tagryn/actions/workflows/verify.yml). Version 0.1.0 ist eine lokal ausführbare Entwicklungsfassung mit echten Schreibvorgängen, keine freigegebene plattformübergreifende Produktionsversion. Die folgenden Ergebnisse unterscheiden ausdrücklich zwischen ausgeführten Prüfungen, vorbereiteten Tests und noch fehlender Abnahme.

## Umgebung

- MacBook Pro, Modell MacBookPro18,3, Apple M1 Pro mit zehn CPU-Kernen (acht Performance-, zwei Effizienzkerne), 32 GiB RAM.
- macOS 26.6.2, Build 25G83; Xcode 26.6, Build 17F113.
- Node.js 26.8.2, npm 11.19.1, Rust 1.98.1; Release-Build für `aarch64-apple-darwin`.
- ExifTool 13.59 und privat gebautes Perl 5.44.0; keine Abhängigkeit von einem vorhandenen System-Perl.
- Lokale SSD, warme Betriebssystem-Caches. Die Messungen sind keine universellen Leistungszusagen.

## Tatsächlich ausgeführte Funktionsprüfungen

| Prüfung                  | Ergebnis und Grenze                                                                                                                                                                                                                      |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Produktionsbuild         | Vue-Typecheck, Vite-Produktionsbuild und nativer Tauri-Release-Build erfolgreich                                                                                                                                                         |
| Rust                     | `cargo check` und Formatprüfung erfolgreich; nicht gleichbedeutend mit einer bestandenen vollständigen Testsuite                                                                                                                         |
| Abhängigkeiten           | `npm audit --omit=dev` meldete zum Prüfzeitpunkt keine bekannten Schwachstellen; keine umfassende Sicherheitsprüfung                                                                                                                     |
| Nativer JPEG-Durchstich  | Datei in der gebauten App geöffnet, Titel über die echte Oberfläche vorgemerkt, Diff geprüft, gesichert gespeichert und unabhängig mit ExifTool erneut gelesen                                                                           |
| Native Wiederherstellung | Wiederherstellung über Jobhistorie und ausdrücklichen Bestätigungsdialog; SHA-256 der wiederhergestellten Datei identisch mit dem Original: `6eec55ddd64145050596b0a52cc637b94d7c984d086737e092c67f1966f0218d`                           |
| Rust-Service-Durchstich  | Unicode-Dateiname, mehrzeiliger Titel mit literalem `$(...)`, echter Schreibplan, Backup, erneutes Lesen und bytegleiche Wiederherstellung erfolgreich; erster Lesezugriff 78 ms, Schreiben und Verifizieren 169 ms in diesem Einzellauf |
| Sidecars                 | Gezielt ausgeführter Test für Neuanlage, Konflikte und Kontaktfelder erfolgreich                                                                                                                                                         |
| Zeitzonen                | Gezielt ausgeführter Test: vorhandenen EXIF-Offset beim Verschieben erhalten, fehlende Zeitzone nicht erfinden; erfolgreich                                                                                                              |
| Standortbereinigung      | Gezielt ausgeführter Test mit anschließendem Lesen; Orientierung, dekodierte Pixel und tatsächlich vorhandenes ICC-Profil erhalten                                                                                                       |
| Teilfehler im Batch      | Gezielt ausgeführter Test bestätigt unabhängigen Erfolg und Fehler verschiedener Dateien                                                                                                                                                 |
| Import                   | Einzelner Node-Test für Feldzuordnung und Unicode erfolgreich                                                                                                                                                                            |
| Icon-Container           | Einzelner Node-Test für ICNS-/ICO-Struktur erfolgreich                                                                                                                                                                                   |
| Private Runtime          | Gesamte Runtime in ein temporäres Verzeichnis kopiert, mit unbrauchbarem PATH gestartet; Version und echtes Kamera-Tag gelesen                                                                                                           |
| Zwei Kameraoriginale     | ARW und NEF gelesen, eingebettete Vorschau erzeugt, Sidecar geschrieben und wiederhergestellt; Originalbytes unverändert                                                                                                                 |

Alle Schreibprüfungen nutzten synthetische Dateien oder ausdrücklich angelegte Wegwerfkopien. Normale Benutzereinstellungen blieben bei der nativen Abnahme durch eigene `TAGRYN_PROFILE_DIR`-Verzeichnisse getrennt. Backups der Abnahme sind keine Anwendungsbeispiele für echte Nutzerdaten.

## Oberfläche und Barrierefreiheit

`scripts/visual-check.mjs` prüfte den isolierten Chromium-Reviewmodus mit zuvor vom echten Rust-/ExifTool-Service gelesenen Daten. Dieser Modus führt keine Schreibvorgänge aus und wird nicht in den Produktionsbuild eingebaut.

Geprüft wurden helle und dunkle Darstellung, 1440 × 960, 1000 × 720 und 900 × 620 Pixel, maximale Schriftgröße 17 px, Liste, Thumbnail-Raster, Vergleich, vorgemerkte Änderungen, technische Tags, englische Oberfläche und Tastaturbedienung der Command-Palette. Keine JavaScript-Seitenfehler oder horizontalen Überläufe des App-Rahmens. Screenshots und Messwerte liegen unter `artifacts/screenshots/`.

Die große Schrift deckte zunächst 14 abgeschnittene Tag-Werte auf. Eine reproduzierbare Geometrieprüfung zeigte veraltete Zeilenhöhen nach dem Wechsel der technischen Ansicht. Der Inspector invalidiert jetzt die Virtualisierungsmaße bei Darstellungs- und Schriftgrößenänderungen. Dieselbe Prüfung meldet anschließend null abgeschnittene Werte; der ergänzte Playwright-Regressionsfall wurde einzeln ausgeführt und bestand (763 ms). Er gehört außerdem zur vorbereiteten CI. Das ist eine Prüfung sichtbarer Wertzeilen, keine Zusicherung, dass beliebig lange Inhalte ohne Abschneiden vollständig in einer Zeile erscheinen.

Die tatsächliche macOS-App wurde zusätzlich über ihre Accessibility-Steuerelemente bedient: Bearbeiten, Vormerken, Schreiben, Jobhistorie, Wiederherstellen, native Bestätigung und kontrolliertes Beenden. Am finalen Release-Bundle wurden außerdem beide Farbschemata über die echten Einstellungen gewechselt und als `artifacts/screenshots/native-light.png` und `native-dark.png` aufgenommen. Die Automation berücksichtigt ersetzte Accessibility-Nachfahren und WebKits Checkbox-Rolle für gedrückte Umschaltbuttons; dafür wurde kein Produktcode verändert. Browserprüfung und native Abnahme sind bewusst getrennte Nachweise. Ein vollständiger VoiceOver-/NVDA-Durchlauf, alle Tastaturpfade und sämtliche Dialoge bei jeder Schriftgröße sind noch nicht abgenommen.

## Leistung

### Nativer Start und Speicher

`artifacts/native-timing.json` enthält drei Starts der gebauten App mit einem echten JPEG-Fixture, je neuem SQLite-Profil, warmen OS-Caches und `PATH=/nonexistent`. Der Messprozess aktiviert das Fenster nach 150 ms; diese Verzögerung ist enthalten. „Bereit“ bezeichnet das initialisierte UI mit erstem sichtbarem Frame, nicht den Abschluss sämtlicher Metadatenabfragen.

Die Schlussmessung am finalen Bundle ergab 735, 669 und 512 ms, Median **669 ms**. Der Hauptprozess lag im Median bei etwa **118,7 MiB RSS**, Hauptprozess und zwei direkte Perl-Kinder zusammen bei etwa **162,3 MiB RSS**. Gemeinsam genutzte beziehungsweise XPC-WebKit-Prozesse fehlen in dieser Zuordnung: **Dies ist nicht der gesamte Speicherverbrauch der App.** Der JSON-Nachweis wird bei erneuter Messung überschrieben.

Ein zunächst nicht aktiviertes Hintergrundfenster verzögerte den `requestAnimationFrame`-Bereitschaftsmarker und ließ den Messprozess auslaufen. Getrennte Profile und ein direkter Runtime-Test isolierten die Ursache. Explizite Aktivierung sowie Beenden über den nativen Fensterknopf ergaben drei regulär abgeschlossene Messläufe. Diese Korrektur betrifft den Messaufbau und ist kein Beleg für eine beschleunigte Engine.

### Scan, Metadaten und Vorschau

| Datensatz / Operation                                                   | Gemessen                                                                |
| ----------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| 10.000 echte Verzeichniseinträge, Hardlinks auf synthetisches JPEG      | 550 ms Scan; etwa 18.182 Einträge/s                                     |
| 200 frische Engine-Lesevorgänge desselben JPEG-Inhalts, jeweils 88 Tags | 1411 ms; etwa 142 Dateien/s                                             |
| Generiertes 24-Megapixel-TIFF, 1.402.834 Byte                           | 19 ms Metadaten; 255 ms Vorschau                                        |
| Sony ILCE-7M5 ARW, 23.232.512 Byte, 323 Tags                            | 335 ms erstes Lesen; 288 ms Vorschau; zehn frische Lesevorgänge 2429 ms |
| Nikon D2X NEF, 12.605.404 Byte, 205 Tags                                | 183 ms erstes Lesen; 215 ms Vorschau; zehn frische Lesevorgänge 1314 ms |

Nachweise: `artifacts/benchmark.json` und `artifacts/camera-check.json`. Hardlinks und synthetische JPEGs sind **nicht repräsentativ für eine gemischte Bibliothek aus 10.000 Kameraoriginalen**. Die zwei echten RAW-Dateien ergänzen diesen Belastungstest, ersetzen aber keine umfangreiche Kamera-/Formatmatrix. Scan-Durchsatz bedeutet Verzeichniseinträge erfassen, nicht alle Metadaten vollständig lesen.

Der UI-Test mit dem tatsächlichen 10.000-Einträge-Scan hielt höchstens 24 Dateizeilen im DOM. In 100 Scroll-Frames: Median 16,6 ms, P95 17,6 ms, Maximum 20,3 ms; letzte Datei erreichbar, kein horizontaler Überlauf bei 900 px, keine Seitenfehler. Dies wurde in **isoliertem Chromium**, nicht als nativer WebKit-FPS-Benchmark gemessen. Nachweis: `artifacts/screenshots/10000-files.json`.

### Provenienz der Kameraoriginale

Die beiden einzelnen Beispieldateien stammen aus der als CC0 ausgewiesenen Sammlung [raw.pixls.us](https://raw.pixls.us/); keine vollständige Sammlung wurde gespiegelt. Das Verzeichnis ist über den [offiziellen JSON-Katalog](https://raw.pixls.us/json/getrepository.php?set=all) nachvollziehbar.

- [Sony ILCE-7M5, Katalogeintrag 8846](<https://raw.pixls.us/getfile.php/8846/nice/Sony%20-%20ILCE-7M5%20-%2014bit%20(3:2).ARW>): SHA-256 `4e3a99cbcf2a348decaf3ca68c5de22b7a05eaa590f4e9184735c1f9c45213a7`.
- [Nikon D2X, Katalogeintrag 9065](<https://raw.pixls.us/getfile.php/9065/nice/Nikon%20-%20D2X%20-%2012bit%20compressed%20(Lossy%20(type%201))%20(3:2).NEF>): SHA-256 `b0b96cc953b8c2f86a216e64e575cbdaee935ddfe75e5a4adc861e41f668b83b`.

Die RAW-Dateien liegen nur im ignorierten lokalen Prüfverzeichnis; sie werden weder in die App eingebettet noch als notwendige CI-Fixtures vorausgesetzt.

## Branding und Bundle

Drei editierbare Vektorentwürfe und die ausgearbeitete Kombination aus Bildrahmen und Detailkante sind in `assets/brand/` dokumentiert. Die Specimen-Tafel zeigt helle/dunkle Hintergründe und kleine Größen einschließlich 16 und 32 Pixeln. Optisch vereinfachte kleine Varianten und größere PNGs, ICNS und ICO werden durch `scripts/build-icons.mjs` reproduzierbar erzeugt. Für diese code-native Vektorgestaltung wurden keine generativen Rasterwerkzeuge benötigt.

`scripts/verify-bundle.mjs --adhoc-sign` kontrollierte am tatsächlichen macOS-Bundle Produktname, Bundle-ID, Icon-Hash, genau ein ausgeliefertes App-Programm und alle 53 enthaltenen Mach-O-Dateien auf eine Mindestversion von höchstens macOS 13.3. Die Laufzeit startete im Bundle ohne brauchbaren System-PATH. Die anschließende tiefe, strikte Codesign-Prüfung war erfolgreich. Nachweis: `artifacts/bundle-check.json`.

Die Signatur ist **nur ad hoc**, nicht Developer-ID-signiert und nicht notarisiert. Windows-/Linux-Pakete und macOS Intel sind vorbereitet, aber nicht lokal gebaut oder auf diesen Systemen getestet. Keine Rechts-, Domain- oder Markenfreigabe des Namens wurde behauptet.

Das lokale Paket `artifacts/Tagryn-0.1.0-macos-arm64.zip` umfasst 30.187.270 Byte. Die Archivprüfung meldete keine Fehler. Nach dem Entpacken in ein anderes temporäres Verzeichnis bestanden erneut die tiefe Signaturprüfung und der ExifTool-Start ohne System-PATH. Alle 53 Mach-O-Dateien wurden außerdem auf absolute Bibliotheksverweise außerhalb von `/System/` und `/usr/lib/` geprüft; keine gefunden. SHA-256: `a6cb509d1528b478a129c445cc6c52ce9514cbca593a6a08a74ef463ca29333d`. Es enthält ausschließlich das App-Bundle; Quellcode, Prüfdateien und Kameraoriginale gehören nicht zum Installationspaket.

## Noch offene Abnahme

Die vollständige Suite darf gemäß Projektvorgabe nur in GitHub Actions laufen und wurde lokal **nicht** ausgeführt. Die CI-Matrix ist vorbereitet. Zum damaligen Prüfzeitpunkt gab es noch kein GitHub-Repository und keinen bestandenen CI-Lauf.

Für die CI vorhandene, lokal noch nicht ausgeführte Integritätsfälle umfassen insbesondere beschädigte Dateien, fehlende Schreibrechte, nicht freigegebene Dateipfade, doppelte Tags, externe Änderungen, manipulierte Backups und Batch-Abbruch. Ähnliche Pfade wurden zum Teil über den Durchstich berührt; das ersetzt nicht diese Regressionstests.

Weitere Freigabekriterien: saubere Neuinstallation auf allen Zielplattformen, Gatekeeper/SmartScreen, echte HEIC/AVIF/DNG/TIFF-/Audio-/Video-/PDF-Formatmatrix, Fehler bei knappem Speicherplatz und Prozessabbruch, größere gemischte RAW-Bibliothek, vollständiger nativer Speicherverbrauch und WebKit-Scrollmessung, Screenreader-Abnahme und Distributionssignierung. Die [bewusst nicht implementierten Funktionen](FORMATS.md) sind davon getrennte Produktgrenzen.

## Reproduktion ausgewählter Prüfungen

Nach `npm ci`, Runtime- und Fixture-Erzeugung lassen sich einzelne Durchstiche gezielt ausführen:

```sh
cargo run --manifest-path src-tauri/Cargo.toml --release --example smoke --locked
cargo run --manifest-path src-tauri/Cargo.toml --release --example benchmark --locked
node scripts/verify-runtime.mjs
node scripts/verify-bundle.mjs --adhoc-sign
```

Bei laufendem `npm run dev` prüfen `node scripts/visual-check.mjs` und `node scripts/large-ui-check.mjs` die separat erzeugten Reviewdaten. Die native Messung `node scripts/measure-native.mjs 3` setzt das fertig gebaute Bundle und macOS-Bedienungshilfenrechte für den ausführenden Prozess voraus; sie aktiviert und schließt nur die eigens gestartete Tagryn-App. Nicht parallel zu einer eigenen offenen Tagryn-Sitzung ausführen. Die vollständigen Testbefehle stehen ausschließlich im CI-Workflow.

# Tagryn — deutsche Anleitung

[Herunterladen & installieren](#herunterladen-und-installieren) · [English overview](README.md) · [Mitmachen](CONTRIBUTING.md) · [Projektprinzipien](docs/PRINCIPLES.md)

**Your files. Every detail.** Eine lokale Desktop-Werkbank zum Untersuchen, Vergleichen und gesicherten Bearbeiten von Metadaten. Tauri 2, Rust, Vue 3 und ExifTool; ohne Cloudkonto, Telemetrie oder automatische Kartenabfragen.

![Tagryn: drei Icon-Entwürfe und die ausgearbeitete Identität](assets/brand/specimen.png)

## Entwicklungsstand

Dies ist eine ausführbare Version **0.1.0**, keine reine Mockup-Oberfläche. Dateizugriff, Metadaten, Schreibpläne, Backups, Wiederherstellung und Jobhistorie verwenden den echten Rust-Service und das mitgelieferte ExifTool. Der veröffentlichte Stand `v0.1.0-alpha.1` hat [Tests und Paketbau auf allen vier CI-Plattformen bestanden](https://github.com/franzgollhammer/tagryn/actions/runs/34597693265). Die Abnahme auf frischen Endnutzer-Systemen und die Distributionssignierung stehen noch aus. Arbeiten Sie zunächst mit Kopien. Die Anwendung verspricht keine vollständige Anonymisierung.

## Herunterladen und installieren

Der [Release `v0.1.0-alpha.1`](https://github.com/franzgollhammer/tagryn/releases/tag/v0.1.0-alpha.1) enthält derzeit **nur Quellcode**. Die GitHub-Downloads „Source code (zip)“ und „Source code (tar.gz)“ sind keine Installationsprogramme.

Zum frühen Ausprobieren gibt es im [erfolgreichen Build dieses Releases](https://github.com/franzgollhammer/tagryn/actions/runs/34597693265) folgende temporäre Downloads:

| System                                           | Jetzt verfügbar                | Installation                                                                                                                                                                     |
| ------------------------------------------------ | ------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Windows x64                                      | Artefakt `tagryn-windows-2025` | ZIP entpacken, `src-tauri/target/release/bundle/nsis/Tagryn_0.1.0_x64-setup.exe` öffnen und dem Installer folgen. Anschließend Tagryn im Startmenü öffnen.                       |
| Ubuntu / kompatibles Debian-basiertes Linux, x64 | Artefakt `tagryn-ubuntu-22.04` | ZIP entpacken und das enthaltene `.deb` mit dem folgenden Befehl installieren. Der Build lief auf Ubuntu 22.04; die Installation auf anderen Distributionen wurde nicht geprüft. |
| macOS ab 13.3, Apple Silicon oder Intel          | [Lokal bauen](#macos)          | Die CI-`.app` besteht die Bundle-Signaturprüfung noch nicht. Ein verifiziertes macOS-Installationspaket zum Herunterladen fehlt daher noch.                                      |

Melden Sie sich bei GitHub an, öffnen Sie den verlinkten Lauf und wählen Sie unten unter **Artifacts** das genannte ZIP. Für [Artefakt-Downloads](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/download-workflow-artifacts) ist ein GitHub-Konto erforderlich. Diese Downloads laufen am **25. September 2026** ab. Sind sie nicht mehr verfügbar, verwenden Sie die Anleitung zum lokalen Bauen weiter unten. Dauerhafte Installer sind dem Release noch nicht angehängt.

Für Linux öffnen Sie ein Terminal im entpackten Artefaktordner:

```sh
sudo apt install ./src-tauri/target/release/bundle/deb/Tagryn_0.1.0_amd64.deb
tagryn
```

`apt` installiert die benötigten GTK-/WebKitGTK-Bibliotheken mit. Tagryn lässt sich auch über das Anwendungsmenü starten. Für Fedora, Arch und andere nicht Debian-basierte Systeme ist ein nativer Quellcode-Build nötig. Dieser Release enthält weder RPM, AppImage, Flatpak noch Linux-ARM-Pakete.

Dies sind **unsignierte Entwicklungspakete** mit bestandenen CI-Tests und Builds, aber ohne Installationsabnahme auf frischen Endnutzer-Systemen. Windows kann einen unbekannten Herausgeber oder eine SmartScreen-Warnung anzeigen. Fahren Sie nur fort, wenn Sie diesem Projekt-Download vertrauen; lassen Sie die Schutzfunktionen des Betriebssystems eingeschaltet. Windows benötigt die [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/). Fertige Pakete enthalten ExifTool samt Laufzeit; Node.js, Rust und eine separate Perl-Installation werden nicht benötigt.

## Funktionsumfang

Implementiert sind native Datei- und Ordnerdialoge, Drag-and-drop, rekursive Scans, virtualisierte Liste und Thumbnail-Raster, Volltextsuche in eingelesenen Metadaten, Mehrfachauswahl, Gruppenfilter, Rohwerte und formatierte Werte, technische Tag-Identitäten, Favoriten, eigene Metadatenspalten, gespeicherte Ansichten, Vergleich von bis zu acht Dateien, Einzel- und Stapelbearbeitung freigegebener Felder, Keyword-Operationen, Zeitverschiebung, GPS-Felder, XMP-Sidecars mit Konfliktanzeige, Vorlagen, Feldübertragung, Umbenennungsplanung, CSV/JSON-Export und Import mit Feldzuordnung, Datenschutzpläne und wiederherstellbare Backups.

Die Oberfläche bietet Deutsch/Englisch, Hell/Dunkel/System, skalierbare Schrift, verstellbare Bereiche, sichtbaren Tastaturfokus, native Menüs und Kontextmenüs sowie eine Befehlspalette. Metadatentexte aus der Engine bleiben teilweise Englisch; technische Fehler enthalten bewusst die Originalmeldung.

**Explizit nicht implementiert:** GPX-Zuordnung, Onlinekarten, gemeinsame Umbenennung eines Mediums und seines bereits vorhandenen Sidecars, beliebige technische Schreibfelder oder Gruppenübertragung, eingebettete Audio-/Video-/PDF-Schreiboperationen, vollständig farbverwaltete Vorschauen für alle Formate, Live-Dateisystemüberwachung und automatische Updates. Diese Grenzen werden auch in der App unter „Formate & Grenzen“ angezeigt. Details: [Format- und Synchronisierungsregeln](docs/FORMATS.md).

## Aus dem Quellcode bauen

Bauen Sie auf dem Betriebssystem und der Prozessorarchitektur, auf denen Tagryn laufen soll. Voraussetzungen: Git, Node.js 24 LTS mit npm, Rust über rustup sowie die [Tauri-Systemvoraussetzungen](https://v2.tauri.app/start/prerequisites/). `rust-toolchain.toml` legt Rust 1.98.1 fest.

- macOS: Xcode Command Line Tools, macOS 13.3 oder neuer. Die lokale Abnahme erfolgte auf Apple Silicon; Intel hat einen eigenen CI-Build.
- Windows: Visual Studio Build Tools mit C++-Desktopkomponenten und WebView2. Das mitgelieferte Windows-ExifTool enthält seine Laufzeit bereits.
- Linux: Compiler, make und die GTK/WebKitGTK-4.1-Entwicklungsbibliotheken. Die CI-Datei enthält den konkreten Ubuntu-22.04-Paketbefehl. Eine aktuelle WebKitGTK-Version ist für die modernen CSS-Funktionen erforderlich.

Unter macOS installieren Sie die Kommandozeilenwerkzeuge mit `xcode-select --install`. Unter Ubuntu 22.04 verwendet CI folgende Build-Abhängigkeiten:

```sh
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev librsvg2-dev libayatana-appindicator3-dev patchelf
```

Unter Windows wählen Sie in Visual Studio Build Tools **Desktopentwicklung mit C++** und verwenden eine x64-MSVC-Rust-Toolchain. Öffnen Sie nach der Installation der Voraussetzungen ein neues PowerShell-Fenster. Für andere Linux-Distributionen gelten die jeweiligen Tauri-Voraussetzungen; die `.deb`-Befehle sind nur für Debian-basierte Systeme geeignet.

Laden Sie den veröffentlichten Quellcode herunter und bereiten Sie ihn vor. Diese Befehle funktionieren im macOS-/Linux-Terminal und in Windows PowerShell:

```sh
git clone --branch v0.1.0-alpha.1 --depth 1 https://github.com/franzgollhammer/tagryn.git
cd tagryn
npm ci
npm run runtime
npm run icons
npm run licenses
```

Der erste Runtime-Build benötigt Internet und einen C-Compiler; auf macOS/Linux wird ein privates, verschiebbares Perl gebaut. Spätere App-Starts und sämtliche Kernfunktionen arbeiten offline. Es wird **keine vorhandene Perl- oder ExifTool-Installation** vorausgesetzt. Ein fertiges App-Bundle benötigt weder Node noch Rust.

Führen Sie anschließend die Befehle für Ihr System im Ordner `tagryn` aus. Die erzeugten Dateinamen verwenden Version `0.1.0`; der Git-Tag kennzeichnet den Alpha-Stand.

### macOS

Verwenden Sie ein natives Terminal mit passender Toolchain: Apple Silicon erzeugt eine ARM64-App, Intel eine x64-App. Ein universelles Paket wird nicht gebaut.

```sh
npm run bundle -- --bundles app
mkdir -p artifacts
node scripts/verify-bundle.mjs --adhoc-sign
open src-tauri/target/release/bundle/macos
```

Ziehen Sie nach erfolgreicher Prüfung **Tagryn.app** aus dem geöffneten Finder-Fenster nach **Programme** und starten Sie die App dort. Der Prüfer kontrolliert die eingebettete Laufzeit und erzeugt eine lokale Ad-hoc-Signatur. Sie gilt für die Nutzung am Build-Rechner und ersetzt keine Apple-Developer-ID-Signatur oder Notarisierung zur Weitergabe. Wiederholen Sie die Prüfung nach jedem Build.

### Windows

In PowerShell:

```powershell
npm run bundle -- --bundles nsis
& ".\src-tauri\target\release\bundle\nsis\Tagryn_0.1.0_x64-setup.exe"
```

Folgen Sie dem Installer und öffnen Sie **Tagryn** über das Startmenü. Der Installer ist unsigniert. Windows-ARM64-Pakete werden derzeit nicht gebaut.

### Linux

Unter Ubuntu oder einem kompatiblen Debian-basierten x64-System:

```sh
npm run bundle -- --bundles deb
sudo apt install ./src-tauri/target/release/bundle/deb/Tagryn_0.1.0_amd64.deb
tagryn
```

Auf anderen Distributionen erzeugt `npm run bundle -- --no-bundle` nach Installation der jeweiligen Tauri-Voraussetzungen und der gemeinsamen Vorbereitung die lokale Programmdatei `src-tauri/target/release/tagryn`. Starten Sie diese aus dem Build-Verzeichnis und belassen Sie die erzeugte Laufzeit sowie Ressourcen dort. Dies ist ein lokaler Build, kein Systempaket; eine distributionsspezifische Installationsabnahme fehlt.

### Entwicklung und Updates

`npm run desktop` startet die native App im Entwicklungsmodus ohne Installation. `npm run dev` startet lediglich den UI-Entwicklungsserver. Dateizugriff und Schreiben funktionieren ausschließlich in der Desktop-App. Der optionale Browser-Reviewmodus ist ausdrücklich schreibgeschützt und lädt echte, zuvor von ExifTool gelesene Wegwerf-Fixtures; er ist im Produktionsbuild nicht enthalten.

Tagryn hat keinen automatischen Updater. Schließen Sie die App für ein Update und installieren Sie ein neueres Paket oder bauen Sie dessen Release-Tag in einem frischen Checkout.

Für isolierte Abnahmeprofile kann vor dem Start `TAGRYN_PROFILE_DIR` auf ein eigenes absolutes Verzeichnis gesetzt werden. Dort liegen ausschließlich die betreffenden Einstellungen, Caches und Jobdaten; der normale App-Datenordner bleibt unverändert. Diese Option ersetzt keine Prozess-Sandbox. Beispiel: `TAGRYN_PROFILE_DIR=/absolute/path/to/test-profile npm run desktop`.

## Erster sicherer Arbeitsablauf

1. Öffnen Sie eine JPEG-Kopie über „Dateien öffnen“ oder ziehen Sie sie ins Fenster.
2. Wählen Sie „Bearbeiten“, ändern Sie beispielsweise den Titel und merken Sie die Änderung vor.
3. „Änderungen prüfen“ liest die Datei erneut und zeigt den konkreten Vorher-nachher-Diff einschließlich Schreibziel und Warnungen.
4. „Gesichert speichern“ erstellt eine verifizierte Sicherung, schreibt über eine temporäre Datei und liest das tatsächliche Ergebnis erneut.
5. „Jobs & Wiederherstellung“ zeigt das Ergebnis pro Datei. „Wiederherstellen“ prüft zuerst, ob zwischenzeitlich weitere Änderungen vorgenommen wurden, und bewahrt auch die rückgängig gemachte Version auf.

„Verwerfen“ löscht ausschließlich ungespeicherte Entwürfe. Es verändert keine Datei. „Wiederherstellen“ ist dagegen eine ausdrückliche Dateisystemoperation. Backups liegen neben dem Schreibziel in `.tagryn-backups/<job-id>/` und enthalten weiterhin die ursprünglichen, möglicherweise sensiblen Metadaten. Geben Sie diese Ordner nicht versehentlich weiter.

## Tastatur

| Aktion                   | macOS                            | Windows/Linux                        |
| ------------------------ | -------------------------------- | ------------------------------------ |
| Dateien / Ordner öffnen  | ⌘O / ⇧⌘O                         | Ctrl+O / Ctrl+Shift+O                |
| Suchen / Befehlspalette  | ⌘F / ⌘K                          | Ctrl+F / Ctrl+K                      |
| Bearbeiten / Vergleichen | ⌘E / ⌘D                          | Ctrl+E / Ctrl+D                      |
| Änderungen prüfen        | ⌘S                               | Ctrl+S                               |
| Einstellungen            | ⌘,                               | Ctrl+,                               |
| Dateiauswahl             | Pfeiltasten, Home/End, Shift, ⌘A | Pfeiltasten, Home/End, Shift, Ctrl+A |

Bereichstrenner sind per Tab erreichbar und mit Pfeiltasten verstellbar. Escape schließt Dialoge. Die Command-Palette lässt sich über Suche, Tab und Enter bedienen.

## Projekt und Nachweise

- [Architektur, Versionsentscheidungen und Distribution](docs/ARCHITECTURE.md)
- [Unterstützte Formate, Schreibfelder und Grenzen](docs/FORMATS.md)
- [Sicherheit und Datenintegrität](docs/SAFETY.md)
- [Prüfprotokoll und Leistungsmessungen](docs/VALIDATION.md)
- [Fünf Namen, drei Icon-Entwürfe und finale Assets](docs/BRAND.md)
- [Ursprünglicher Plan mit drei Architekturvarianten](.claude/plans/2026-09-10-tagryn-desktop.md)
- [Fremdlizenzen](docs/THIRD-PARTY-NOTICES.md) und [gesammelte Lizenztexte](docs/DEPENDENCY-LICENSES.txt)

Die vollständige Testsuite ist ausschließlich für [GitHub Actions](.github/workflows/verify.yml) vorgesehen. Lokal werden nur ausdrücklich ausgewählte, relevante Tests und nachvollziehbare Durchstichprüfungen ausgeführt. Der Integrationsbranch ist `develop`; `main` wird nicht verwendet. Ein öffentlicher Quellcode-Alpha-Release ist von signierten Installationspaketen zu unterscheiden.

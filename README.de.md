# Tagryn — deutsche Anleitung

[English overview](README.md) · [Mitmachen](CONTRIBUTING.md) · [Projektprinzipien](docs/PRINCIPLES.md)

**Your files. Every detail.** Eine lokale Desktop-Werkbank zum Untersuchen, Vergleichen und gesicherten Bearbeiten von Metadaten. Tauri 2, Rust, Vue 3 und ExifTool; ohne Cloudkonto, Telemetrie oder automatische Kartenabfragen.

![Tagryn: drei Icon-Entwürfe und die ausgearbeitete Identität](assets/brand/specimen.png)

## Entwicklungsstand

Dies ist eine ausführbare Version **0.1.0**, keine reine Mockup-Oberfläche. Dateizugriff, Metadaten, Schreibpläne, Backups, Wiederherstellung und Jobhistorie verwenden den echten Rust-Service und das mitgelieferte ExifTool. Noch keine allgemein freigegebene Produktionsversion: Die vollständige plattformübergreifende CI-Abnahme und Distributionssignierung stehen aus. Arbeiten Sie zunächst mit Kopien. Die Anwendung verspricht keine vollständige Anonymisierung.

Implementiert sind native Datei- und Ordnerdialoge, Drag-and-drop, rekursive Scans, virtualisierte Liste und Thumbnail-Raster, Volltextsuche in eingelesenen Metadaten, Mehrfachauswahl, Gruppenfilter, Rohwerte und formatierte Werte, technische Tag-Identitäten, Favoriten, eigene Metadatenspalten, gespeicherte Ansichten, Vergleich von bis zu acht Dateien, Einzel- und Stapelbearbeitung freigegebener Felder, Keyword-Operationen, Zeitverschiebung, GPS-Felder, XMP-Sidecars mit Konfliktanzeige, Vorlagen, Feldübertragung, Umbenennungsplanung, CSV/JSON-Export und Import mit Feldzuordnung, Datenschutzpläne und wiederherstellbare Backups.

Die Oberfläche bietet Deutsch/Englisch, Hell/Dunkel/System, skalierbare Schrift, verstellbare Bereiche, sichtbaren Tastaturfokus, native Menüs und Kontextmenüs sowie eine Befehlspalette. Metadatentexte aus der Engine bleiben teilweise Englisch; technische Fehler enthalten bewusst die Originalmeldung.

**Explizit nicht implementiert:** GPX-Zuordnung, Onlinekarten, gemeinsame Umbenennung eines Mediums und seines bereits vorhandenen Sidecars, beliebige technische Schreibfelder oder Gruppenübertragung, eingebettete Audio-/Video-/PDF-Schreiboperationen, vollständig farbverwaltete Vorschauen für alle Formate, Live-Dateisystemüberwachung und automatische Updates. Diese Grenzen werden auch in der App unter „Formate & Grenzen“ angezeigt. Details: [Format- und Synchronisierungsregeln](docs/FORMATS.md).

## Lokal starten

Voraussetzungen: Node.js 24 LTS (mindestens 22.12), npm, Rust über rustup sowie die [Tauri-Systemvoraussetzungen](https://v2.tauri.app/start/prerequisites/). `rust-toolchain.toml` legt Rust 1.98.1 fest.

- macOS: Xcode Command Line Tools, macOS 13.3 oder neuer. Die lokale Abnahme erfolgte auf Apple Silicon; Intel hat einen eigenen CI-Build.
- Windows: Visual Studio Build Tools mit C++-Desktopkomponenten und WebView2. Das mitgelieferte Windows-ExifTool enthält seine Laufzeit bereits.
- Linux: Compiler, make und die GTK/WebKitGTK-4.1-Entwicklungsbibliotheken. Die CI-Datei enthält den konkreten Ubuntu-22.04-Paketbefehl. Eine aktuelle WebKitGTK-Version ist für die modernen CSS-Funktionen erforderlich.

```sh
npm ci
npm run runtime
npm run icons
npm run licenses
npm run desktop
```

Der erste Runtime-Build benötigt Internet und einen C-Compiler; auf macOS/Linux wird ein privates, verschiebbares Perl gebaut. Spätere App-Starts und sämtliche Kernfunktionen arbeiten offline. Es wird **keine vorhandene Perl- oder ExifTool-Installation** vorausgesetzt. Ein fertiges App-Bundle benötigt weder Node noch Rust.

```sh
# macOS: tatsächliches .app-Bundle erzeugen
npm run bundle -- --bundles app

# Windows: NSIS-Installer, auf Windows ausführen
npm run bundle -- --bundles nsis

# Linux: Debian-Paket, auf Linux ausführen
npm run bundle -- --bundles deb
```

Das macOS-Ergebnis liegt unter `src-tauri/target/release/bundle/macos/Tagryn.app`. Lokale Builds sind nicht mit einer Developer-ID signiert oder notarisiert. Die App lässt sich am Build-Rechner aus diesem Ordner starten. Für Weitergabe sind die [Release-Schritte](docs/ARCHITECTURE.md#runtime-und-distribution) erforderlich; Sicherheitsfunktionen des Betriebssystems nicht pauschal deaktivieren.

Optional prüft und signiert `node scripts/verify-bundle.mjs --adhoc-sign` den lokalen macOS-Build einschließlich seiner privaten Perl-Laufzeit. Das ist ausschließlich eine lokale Ad-hoc-Signatur. Der Prüfer kontrolliert außerdem Icon, Bundle-Namen, Mindestversion aller enthaltenen Mach-O-Dateien und ExifTool ohne System-PATH. Nach jedem erneuten Build muss die Prüfung wiederholt werden.

`npm run dev` startet lediglich den UI-Entwicklungsserver. Dateizugriff und Schreiben funktionieren ausschließlich in der Desktop-App. Der optionale Browser-Reviewmodus ist ausdrücklich schreibgeschützt und lädt echte, zuvor von ExifTool gelesene Wegwerf-Fixtures; er ist im Produktionsbuild nicht enthalten.

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

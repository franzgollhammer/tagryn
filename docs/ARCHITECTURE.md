# Architektur und Entscheidungen

## Schichten

```text
Vue-Komponenten → Pinia: Auswahl, Darstellung, ungespeicherte Entwürfe
                             │ kontrollierte Tauri-Kommandos
                             ▼
                       Rust-Service
              ┌──────────────┼────────────────┐
              ▼              ▼                ▼
       Metadatenmodell   Änderungsplaner    Vorschau
              │              │                │
              ▼              ▼                ▼
       ExifTool-Pool ← Jobverwaltung → Dateizugriff/Backups
                             │
                             ▼
                  SQLite: Cache, Suche, Journal,
                    Einstellungen und Vorlagen
```

Dateien und ihre Sidecars bleiben die maßgebliche Quelle. SQLite ist kein proprietärer Metadatenkatalog. Bei einem Schreibplan und vor dem Commit werden die wirklichen Dateien überprüft. Ein unvollständiger Cache oder eine fehlende Vorschau blockieren die Analyse nicht.

| Bereich                   | Zuständigkeit                                                                                   |
| ------------------------- | ----------------------------------------------------------------------------------------------- |
| `src/domain`              | Strikte Typen, kontrollierte UI-Felder, Aggregation, Importvalidierung und Umbenennungsvorschau |
| `src/stores/workspace.ts` | Auswahl, Filter, Ansichten, Entwürfe, begrenzte UI-Caches und Jobereignisse                     |
| `src/components`          | Dateibrowser, Inspector, Vergleich, Tastatur, zugängliche Dialoge und Workflow-Eingaben         |
| `src-tauri/src/model.rs`  | Serialisierte Domänentypen; keine Dateisystemlogik                                              |
| `metadata.rs`             | ExifTool-Ausgabe, eindeutige Tag-Identität, Roh-/Anzeigewerte und Sidecar-Provenienz            |
| `engine.rs`               | Zwei langlebige Prozesse, Request-Zuordnung, Zeitgrenzen und Prozesslebenszyklus                |
| `files.rs`                | Kanonische Pfade, Fingerprints, Formatfähigkeiten und exklusive Dateikopien                     |
| `planner.rs`              | Feld-Allowlist, Validierung, Synchronisierung und exakte Änderungsvorschau                      |
| `jobs.rs`                 | Temporäre Kandidaten, Backup, Commit, Verifikation und Wiederherstellung                        |
| `service.rs`              | Dateiregistrierung, Scans, Abbruch, serialisierte Schreibjobs und Ereignisse                    |
| `preview.rs`              | Begrenzte Decodierung, eingebettete RAW-JPEGs und optionale macOS-Systemkonvertierung           |
| `db.rs`                   | SQLite-WAL, bounded Cache, begrenzter Suchindex und Jobhistorie                                 |

## Metadatenidentität

ExifTool läuft mit `-j -G:0:1:3:4:5:7 -a -D -l -struct -api Struct=2 -api SaveFormat=1`. Gruppe, konkrete IFD/XMP-Gruppe, Dokument, Instanz, Metadatenpfad und Tag-ID bleiben im Schlüssel erhalten. Der Quellentyp `embedded`/`sidecar` ergänzt die Identität. Gleiche Namen wie GPSLatitude oder Artist werden nicht in einer einfachen Namens-Map überschrieben.

`val` enthält den formatierten Wert, `num` den verfügbaren unformatierten Wert, `fmt` die Speicherformatinformation. Fehlt `num`, bleibt der vorhandene Wert erhalten; Tagryn erfindet keinen zweiten Rohwert. `-s` wird ausdrücklich nicht verwendet, weil es die mit `-l` angeforderten Beschreibungen und numerischen Werte unterdrückt. Struktur- und Flattened-Tags bleiben beide sichtbar. Ableitungen aus der Composite-Gruppe sind gekennzeichnet. Mehrdeutige Instanzen desselben beschreibbaren Gruppentags werden nicht stillschweigend gemeinsam editiert.

## Nebenläufigkeit und Grenzen

Zwei ExifTool-Worker bearbeiten je eine Anfrage gleichzeitig. Jede Anfrage erhält eine nummerierte `-execute`-Marke und einen zufälligen Stderr-Endmarker mit Exitstatus. Stdout und Stderr werden gleichzeitig gelesen. Nach 45 Sekunden oder einem Protokollfehler wird der betroffene Worker beendet; die nächste Anfrage startet ihn neu. Schreibanfragen werden nach einem Timeout nicht automatisch erneut ausgeführt.

Das Stay-open-Protokoll ist zeilenorientiert. Schreibwerte werden deshalb byteweise als UTF-8 mit ExifTools `-ec` kodiert; Zeilenumbrüche, führende Leerzeichen und literale Backslashes bleiben erhalten. Unix-Dateinamen mit Zeilenumbrüchen verwenden einen isolierten Prozess mit strukturierten Argumenten. Unter Windows erhalten isolierte Prozesse ihre Argumente über UTF-8-Standardeingabe, damit die System-Codepage keine Zeichen ersetzt. Binäre Vorschaubilder werden ebenfalls isoliert gelesen. NUL bleibt als Eingabe unzulässig. Ausgabe ist pro Stream auf 32 MiB begrenzt, auch vor der vollständigen Zeilenallokation. Benutzerkonfiguration und fremde Perl-Umgebungsvariablen werden deaktiviert. Beim Beenden werden Worker kontrolliert beendet.

Ein Scan läuft außerhalb des UI-Threads und liefert Blöcke von 128 Dateieinträgen; pro Scan gilt eine Grenze von 100.000 Dateien. Versteckte Unterverzeichnisse und verfolgte Symlink-Unterbäume werden ausgelassen. Sichtbare und ausgewählte Dateien werden bevorzugt eingelesen. Ein Preview-Semaphor begrenzt teure Decodierung auf eine gleichzeitige Anfrage. Ein Schreib-Semaphor serialisiert alle Schreib-/Wiederherstellungsjobs.

Die UI behält maximal 128 Metadatendokumente beziehungsweise ungefähr 64 MiB JSON-Text plus bis zu acht angeheftete Auswahl-Dokumente. Dies ist ein Cachebudget, keine harte Grenze des gesamten JavaScript-Heaps. Weitere Zeilen halten nur kompakte Zusammenfassungen. Die Mehrfachübersicht und der Vergleich zeigen maximal acht analysierte Dateien und kennzeichnen diese Teilmenge; Stapelaktionen gelten für alle ausgewählten Dateien. SQLite speichert maximal 128 Dokumente mit jeweils höchstens 2 MiB sowie einen 64-MiB-Suchindex. Backups und das Auditjournal werden nicht automatisch gelöscht. Volltextsuche ist entsprechend auf tatsächlich eingelesene, noch indexierte Dateien begrenzt.

## Verifizierte Versionsbasis, 10. September 2026

| Komponente                | Festgelegter Stand           | Entscheidung                                                                                                                                                     |
| ------------------------- | ---------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tauri Rust / CLI / JS API | 2.11.5 / 2.11.4 / 2.11.1     | Aktuelle stabile 2.x-Familie; unterschiedliche Paketnummern sind normal                                                                                          |
| Vue / Pinia               | 3.5.42 / 4.0.3               | Composition API, strikt typisierter State                                                                                                                        |
| Vite / Vue-Plugin         | 8.2.2 / 6.0.8                | Produktionsbuild geprüft                                                                                                                                         |
| TypeScript / vue-tsc      | 6.0.3 / 3.3.11               | TypeScript 7.0.2 war aktuell, scheiterte aber an der aktuellen vue-tsc-Kompatibilität (`./lib/tsc` nicht exportiert). Bewusst kompatiblen stabilen Stand gewählt |
| Tailwind CSS              | 4.3.3                        | Gemeinsame Tokens und moderne CSS-Basis; macOS mindestens 13.3 wegen Safari/WebKit-Funktionen                                                                    |
| Rust                      | 1.98.1                       | Aktueller stabiler Toolchain-Stand, projektlokal gepinnt                                                                                                         |
| ExifTool                  | 13.59                        | Mitgeliefert, keine PATH-Suche                                                                                                                                   |
| Unix-Perl                 | 5.44.0                       | Eigener verschiebbarer Runtime-Build                                                                                                                             |
| SQLite / rusqlite         | gebündelte SQLite / 0.40.2   | Kein System-SQLite erforderlich                                                                                                                                  |
| Virtualisierung           | TanStack Vue Virtual 3.13.37 | Gemeinsamer virtueller Zeilenmechanismus für Dateien, Tags und Vergleich                                                                                         |

`package-lock.json`, `src-tauri/Cargo.lock` und `rust-toolchain.toml` sind maßgeblich. npm- und Cargo-Registries sowie offizielle Dokumentation wurden vor bzw. während der Integration geprüft. Quellen: [Tauri](https://v2.tauri.app/), [Vue-Releases](https://github.com/vuejs/core/releases), [Vite-Releases](https://github.com/vitejs/vite/releases), [Tailwind-Kompatibilität](https://tailwindcss.com/docs/compatibility), [ExifTool](https://exiftool.org/), [Perl-Distributionen](https://www.cpan.org/src/README.html), [Rust-Releases](https://blog.rust-lang.org/).

## Runtime und Distribution

`scripts/prepare-runtime.mjs` lädt gepinnte ExifTool-Pakete und prüft deren SHA-512. Der Perl-Quellcode wird gegen einen hinterlegten SHA-256 geprüft. macOS/Linux bauen Perl mit `userelocatableinc`; Windows nutzt das originale vendored ExifTool-Paket samt Laufzeit. Für jede Architektur wird ein eigener Runtime-Build benötigt. `npm run verify:runtime` verschiebt die gesamte Runtime in ein temporäres Verzeichnis und startet sie mit unbrauchbarem PATH. Damit wird eine unbemerkte Abhängigkeit vom System-Perl ausgeschlossen.

Die Runtime liegt als Tauri-Bundle-Ressource unter `runtime/`; Rust startet ausschließlich diese aufgelösten Programmpfade. Das ist absichtlich keine Shell-Plugin-Freigabe für beliebige Frontendbefehle. macOS HEIC/AVIF-Vorschauen dürfen zusätzlich das systemeigene `/usr/bin/sips` verwenden. Fehlende Decoder erzeugen eine fehlende Vorschau, keinen Metadaten-Lesefehler.

Die CI-Matrix enthält macOS ARM64, macOS Intel, Windows x64 und Ubuntu x64. Sie baut `.app`, NSIS bzw. `.deb`. Ein separater [Paket-Workflow](../.github/workflows/package-preview.yml) verarbeitet diese geprüften Release-Artefakte zu dauerhaften Alpha-Downloads: ein Apple-Silicon-DMG, ein Intel-Mac-App-ZIP, Windows-NSIS und Linux-Debian. Er prüft Installation beziehungsweise Kopieren, mitgelieferte Lizenztexte, Metadaten-Laufzeit und Frontend-Start auf nativen Runnern. Die macOS-Bundles erhalten vollständige Ad-hoc-Signaturen und lassen die optionalen Perl-Bindings `DB_File`, `GDBM_File` und `NDBM_File` weg, damit keine Homebrew-Datenbankbibliotheken vorausgesetzt werden. Ein Guard prüft, dass ExifTool diese Module nicht referenziert. Details und Provenienz: [Release-Prozess](RELEASING.md#preview-installers).

Diese Konfiguration ersetzt keinen bestandenen Lauf. Eine universelle macOS-Binärdatei oder Windows-ARM64-Pakete werden noch nicht erzeugt.

Alpha-Vorschaupakete kennzeichnen die noch fehlende Herausgebersignierung und Notarisierung ausdrücklich. Für eine Produktionsdistribution mit verifizierter Herausgeberidentität sind erforderlich:

1. Vollständige CI einschließlich echter Runtime-Verifikation auf allen Zielsystemen erfolgreich ausführen.
2. Sämtliche ausführbaren Perl-Dateien und nativen Erweiterungen im macOS-Bundle von innen nach außen mit derselben Developer-ID und passenden Hardened-Runtime-Einstellungen signieren; anschließend die App signieren, notarieren und das Ticket anheften. Ein ad-hoc signierter lokaler Build ist kein Distributionsnachweis.
3. Windows-App und Installer mit einer vertrauenswürdigen Codesigning-Identität signieren. Linux-Pakete samt Prüfsummen in einem nachvollziehbaren Release veröffentlichen.
4. Lizenztexte, Provenienz, Runtime-Hashes, Gatekeeper/SmartScreen-Verhalten und Neuinstallation auf sauberen Systemen kontrollieren.
5. ExifTool-/Perl-Updates als gemeinsame Runtime-Version behandeln: Hashes bewusst aktualisieren, Fixtures/Schreibtests wiederholen, Bundles neu bauen und signieren. Automatische Downloads oder Updates innerhalb der laufenden App gibt es nicht.

Anleitung: [Tauri-Bundling](https://v2.tauri.app/distribute/), [macOS-Signierung](https://v2.tauri.app/distribute/sign/macos/), [Windows-Signierung](https://v2.tauri.app/distribute/sign/windows/).

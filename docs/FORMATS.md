# Formate, Schreibfelder und Grenzen

## Fähigkeiten getrennt betrachten

| Format                                                                                    | Metadaten lesen                             | In dieser Version schreiben                                                                                      | Medienvorschau                                                              |
| ----------------------------------------------------------------------------------------- | ------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| JPEG                                                                                      | ExifTool                                    | Freigegebene XMP-Felder; explizite EXIF/IPTC-Synchronisierung; Datenschutz                                       | Ja, begrenzt decodiert                                                      |
| TIFF                                                                                      | ExifTool                                    | Wie JPEG, sofern ExifTool den konkreten Container unterstützt                                                    | Ja; sehr große oder spezielle TIFFs können ohne Vorschau bleiben            |
| PNG, WebP                                                                                 | ExifTool                                    | Freigegebene XMP-Felder und kontrollierte Bereinigung; keine Legacy-Synchronisierung                             | Ja                                                                          |
| HEIC/HEIF, AVIF                                                                           | ExifTool                                    | Freigegebene XMP-Felder nur bei erfolgreicher Kandidaten-Verifikation; keine Garantie für jede Containervariante | macOS-Systemdecoder, soweit verfügbar; Windows/Linux derzeit keine Vorschau |
| DNG, CR2/CR3, NEF/NRW, ARW/SR2/SRF, RAF, ORF, RW2, PEF, RWL, 3FR, IIQ, KDC, MOS, MRW, X3F | Soweit ExifTool die konkrete Variante liest | Ausschließlich XMP-Sidecar; keine eingebettete RAW-Bereinigung                                                   | Eingebettetes JPEG, sofern vorhanden; keine RAW-Entwicklung                 |
| XMP                                                                                       | Ja                                          | Unterstützte Felder und kontrollierte Bereinigung im XMP-Dokument                                                | Keine Bildvorschau                                                          |
| MP4, MOV, M4V, MKV, AVI                                                                   | Containerspezifische ExifTool-Metadaten     | Keine eingebettete Bearbeitung; ein ausdrücklich gewähltes Sidecar kann Zusatzdaten aufnehmen                    | Keine                                                                       |
| MP3, M4A, WAV, FLAC, OGG                                                                  | Verfügbare Tags über ExifTool               | Wie Video                                                                                                        | Keine                                                                       |
| PDF                                                                                       | ExifTool                                    | Keine eingebettete Bearbeitung/Bereinigung; insbesondere kein irreführendes inkrementelles „Anonymisieren“       | Keine                                                                       |

Die Dateiendung entscheidet nicht allein über Schreibfähigkeit: Der tatsächlich erkannte ExifTool-Dateityp wird mitgeprüft. RAW-Endungen erzwingen den konservativen Sidecar-Weg. Lesefehler, fehlende Rechte und nicht unterstützte Container werden pro Datei ausgewiesen. Ein als grundsätzlich schreibbar aufgeführtes Format ist keine Zusage für jede Variante; erst die Verifikation des temporären Kandidaten erlaubt den Commit. HEIC/AVIF und viele RAW-Varianten benötigen noch zusätzliche formatbezogene CI-Fixtures.

## Freigegebene Felder

| Benutzerfeld             | Standard-Schreibziel                                                      | Typ                                                               |
| ------------------------ | ------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| Titel                    | `XMP-dc:Title`                                                            | Text                                                              |
| Beschreibung             | `XMP-dc:Description`                                                      | Mehrzeiliger Text                                                 |
| Keywords                 | `XMP-dc:Subject`                                                          | Liste: ersetzen, hinzufügen, gezielt entfernen                    |
| Urheber                  | `XMP-dc:Creator`                                                          | Liste                                                             |
| Copyright                | `XMP-dc:Rights`                                                           | Text                                                              |
| Bewertung                | `XMP-xmp:Rating`                                                          | Ganzzahl −1 bis 5                                                 |
| E-Mail, Telefon, Website | `XMP-iptcCore:CreatorWorkEmail`, `CreatorWorkTelephone`, `CreatorWorkURL` | Strukturierte IPTC-Kontaktfelder über freigegebene Flattened-Tags |
| Aufnahmezeit             | `XMP-exif:DateTimeOriginal`                                               | Validiertes Datum mit optionalem unverändertem Offset             |
| Breite, Länge, Höhe      | `XMP-exif:GPSLatitude`, `GPSLongitude`, `GPSAltitude`                     | Zahlen; Breite −90…90, Länge −180…180                             |

Andere Tags sind lesbar, aber nicht beliebig editierbar. Abgeleitete Tags, unbekannte Schreibziele und mehrdeutige Instanzen werden nicht als sicher schreibbar behandelt. Listen werden als einzelne Argumente an ExifTool übergeben, nicht durch selbstgebaute Shell-Kommandos. Ein Wert ist auf 32 KiB begrenzt; NUL-Zeichen werden abgewiesen.

## EXIF/IPTC/XMP-Synchronisierung

Standard ist **nur XMP**. Abweichende eingebettete EXIF/IPTC-Werte werden nicht heimlich geändert. Für JPEG/TIFF kann ausdrücklich synchronisiert werden:

| Feld         | Zusätzliches Legacy-Ziel                                                                                |
| ------------ | ------------------------------------------------------------------------------------------------------- |
| Titel        | `IPTC:ObjectName`                                                                                       |
| Beschreibung | `IFD0:ImageDescription`, `IPTC:Caption-Abstract`                                                        |
| Urheber      | `IFD0:Artist`, `IPTC:By-line`                                                                           |
| Copyright    | `IFD0:Copyright`, `IPTC:CopyrightNotice`                                                                |
| Keywords     | `IPTC:Keywords`                                                                                         |
| Aufnahmezeit | `ExifIFD:DateTimeOriginal`; vorhandener/ausdrücklich angegebener Offset in `ExifIFD:OffsetTimeOriginal` |

IPTC wird als UTF-8 markiert. Feldbezogene Byte-Limits werden vor dem Schreiben geprüft; zu lange Werte werden nicht unbemerkt abgeschnitten. Listen in `IFD0:Artist` werden mit `; ` verbunden. Bewertung, Kontakt- und GPS-Eingaben werden auch bei aktivierter Legacy-Synchronisierung nur in XMP geschrieben. Vorhandene EXIF-GPS-Werte können deshalb abweichen; zur Entfernung dient der gruppenübergreifende Datenschutzplan.

Ein Zeitversatz verändert die Aufnahmezeit um eine explizite Zahl Sekunden. Ein vorhandener EXIF-Offset wird für die XMP-Zeit übernommen, sofern nicht bereits eine XMP-Zeit mit eigener Zeitzone vorliegt. Fehlt eine Zeitzone, bleibt die verschobene Zeit zonenlos. Es gibt keine automatische Interpretation der Rechnerzeitzone und keine stillschweigende Sommerzeitkorrektur. Kamerauhren werden durch eine ausgewählte Dateigruppe und einen gemeinsamen Versatz angeglichen.

## Sidecars und Konflikte

Ein benachbartes `.xmp` bzw. `.XMP` wird zusätzlich gelesen. Eingebettete und Sidecar-Werte behalten ihre Herkunft und sind im Inspector separat sichtbar; abweichende freigegebene Felder erscheinen als Konflikt. Für unterstützte Arbeitsfelder bevorzugt die UI vorhandene Sidecar-Werte. Die Anzeige ist keine automatische Zusammenführung.

RAW-Bearbeitung schreibt bevorzugt und ausschließlich ins Sidecar. Auf Wunsch kann auch für andere Medien ein Sidecar gewählt werden. Fehlende Sidecars werden kontrolliert neu angelegt. Ein Sidecar kann vorhandene sensible Angaben im Original **nicht entfernen**. Eine Bereinigung des eingebetteten Mediums verändert ein vorhandenes Sidecar nicht; der Plan warnt ausdrücklich. Öffnen und bereinigen Sie dieses gegebenenfalls separat.

## Vergleich, Übertragung, Umbenennung und Import

Der Vergleich verbindet technische Tag-Identitäten einschließlich Herkunft, Gruppe und Instanz. „Nur Unterschiede“ blendet identische Werte aus; fehlende Werte bleiben als `∅` erkennbar. Für Lesbarkeit und begrenzten Speicher werden bis zu acht Dateien gleichzeitig verglichen. Die Mehrfachübersicht unterscheidet identisch, unterschiedlich, vollständig fehlend und teilweise fehlend. Bei größerer Auswahl ist ausdrücklich angegeben, dass die Übersicht nur die analysierte Teilmenge beschreibt; Stapelaktionen gelten trotzdem für die gesamte Auswahl.

Die Feldübertragung bietet gezielt die freigegebenen XMP-Felder. Sie kopiert keine beliebigen MakerNotes-, ICC- oder anderen binären Gruppen. Fehlende Quellwerte werden als Entfernung im Plan sichtbar. Vorlagen speichern Feldoperationen in SQLite-Einstellungen und werden beim Anwenden erneut geplant.

Umbenennungen unterstützen `{name}`, `{date}`, `{seq}`, `{seq:3}` und `{ext}`. Erweiterungen müssen erhalten bleiben. Ungültige portable Dateinamen, reservierte Windows-Namen und Zielkollisionen werden abgefangen. Fehlende Aufnahmezeiten werden nicht durch das Änderungsdatum ersetzt. Dateien mit bereits vorhandenem Begleit-Sidecar werden noch nicht gemeinsam umbenannt; der Plan lehnt sie ab. Namenszyklen und ein Austausch zweier vorhandener Namen sind bewusst nicht freigegeben.

JSON-Export erhält vollständige Metadatendokumente unter dem Schema `tagryn.metadata/v1`. CSV ist eine lange Tabelle mit einer Zeile pro Tag, einschließlich Quelle, Gruppe, Tag-Identität, Rohwert und Anzeigewert. Excel-Formelpräfixe werden beim Export entschärft. Der Import akzeptiert Tagryn-JSON oder zeilenbasierte CSV/JSON-Dateien mit einer Datei-Spalte und ausdrücklich zugeordneten Bearbeitungsfeldern. Ein langer CSV-Vollauszug muss vor dem Import in eine solche breite Tabelle umgeformt werden; er ist derzeit kein automatisches Roundtrip-Format. Für einen vollständigen exportierten Datensatz verwenden Sie JSON.

Importe sind auf 16 MiB und 10.000 Zeilen begrenzt. Dateien müssen bereits geöffnet und über einen vollständigen Pfad oder einen eindeutigen Dateinamen auflösbar sein. Mehrdeutige Namen, doppelte Zeilen, doppelte Feldzuordnungen, verschachtelte Schreibobjekte und ungültige Zahlen werden vor der Änderung abgewiesen. Die abschließende Rust-Planung validiert erneut. Export überschreibt keine bestehende Datei; wählen Sie einen neuen Namen.

## Datenschutz

„Standort entfernen“ berücksichtigt bekannte GPS- und Ortsfelder über Metadatengruppen hinweg. „Persönliche Angaben entfernen“ und „Für Veröffentlichung vorbereiten“ erweitern die kontrollierte Liste. MakerNotes und eingebettete Vorschau-/Thumbnail-Blöcke können zusätzliche sensible Daten tragen; der Plan zeigt ihre Entfernung ausdrücklich an. Orientierung und ICC-Profil werden standardmäßig erhalten und erneut verglichen.

Das betrifft bekannte Metadaten und bekannte Vorschautags, nicht beliebige Steganografie, erkennbare Bildinhalte, jeden möglichen unbekannten Herstellerblock oder externe Kataloge. Es gibt keine vollständige Anonymisierungszusage. Backups, Dateinamen, XMP-Begleiter, Exporte und der lokale Suchindex können weiterhin sensible Angaben enthalten. Onlinekarten sind nicht implementiert; GPS-Daten verlassen die App nicht automatisch.

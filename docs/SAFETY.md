# Datenintegrität und Sicherheitsgrenzen

## Schreibprotokoll

1. Die Oberfläche sammelt Entwürfe mit dem beim Lesen erfassten SHA-256, Dateiumfang, nanosekundengenauem Änderungszeitstempel und separatem Sidecar-Fingerprint. Zeitstempel werden als Zeichenketten über IPC transportiert, damit JavaScript keine Präzision verliert.
2. Rust liest für den Plan erneut und prüft Fingerprints, Rechte, tatsächlichen Dateityp, freigegebene Felder, Werte und Ziele. Der Plan erhält eine nur serverseitig ausführbare ID. Die Oberfläche kann keine beliebigen Prozessargumente oder Dateipfade als Schreibplan einschleusen.
3. Ein einzelner Schreibjob erhält den globalen Schreib-Semaphor. Pro Datei werden Quelle und Sidecar nochmals geprüft. Abbruch startet keine weitere Datei; ein bereits laufender Schreibvorgang wird kontrolliert zu Ende geführt.
4. Die Quelle wird exklusiv in eine temporäre Geschwisterdatei kopiert. ExifTool verändert ausschließlich diesen Kandidaten mit strukturierten Prozessargumenten. Es findet keine erneute Bildkompression durch Tagryn statt; Vorschauen sind separate Cachebilder.
5. Der Kandidat wird erneut gelesen. Jede geplante Operation muss nachweisbar sein. Bei Bereinigung werden Orientierung und ICC-Tags mit der Quelle verglichen. Ein fehlgeschlagener Kandidat wird verworfen; das Original bleibt unverändert.
6. Vor dem Commit entsteht eine exklusive, byteverifizierte Sicherung in `.tagryn-backups/<job-id>/`. Der SQLite-Job enthält bereits vor dem Dateiaustausch Backup-Pfad, dessen SHA-256, beabsichtigten Ausgabehash und Originalpfad.
7. Nach erneuter Konfliktprüfung erfolgt der Dateiaustausch. Unix verwendet für dasselbe Ziel `rename`; neue Dateinamen werden ohne Überschreiben bestehender Ziele angelegt. Unter Windows wird die vorherige Version zunächst zurückbehaltbar verschoben; dies ist kein atomarer plattformübergreifender Austausch.
8. Die tatsächlich gespeicherte Datei wird gehasht und nochmals von ExifTool gelesen. Erfolg, Warnung, Fehler oder Überspringen wird pro Datei protokolliert. Teilerfolge bleiben Teilerfolge; es gibt keine Batch-Atomarität.

## Wiederherstellung

Automatische Wiederherstellung erfordert einen passenden Ausgabehash und einen unveränderten, aufgezeichneten Backuphash. Neuere Änderungen durch andere Programme und manipulierte Backups führen zum Abbruch. Die rückgängig gemachte Version wird zuvor in einem zusätzlichen Wiederherstellungsordner bewahrt. Bei einem neu angelegten Sidecar stellt die Aktion dessen vorherige Abwesenheit wieder her, bewahrt aber den entfernten Sidecar-Inhalt als Datei im Wiederherstellungsordner.

Bei Prozessabbruch werden zuvor aktive Jobs beim nächsten Start als `interrupted` geladen, nicht als erfolgreich. Falls der Commit noch nicht erfolgt ist, kann die Datei weiterhin dem Original entsprechen; eine automatische Wiederherstellung darf dann wegen des fehlenden Ausgabehash-Matches ablehnen. Das Journal zeigt den Backup-Pfad für eine manuelle, bewusste Wiederherstellung. Nach hartem Systemabbruch können temporäre Geschwisterdateien verbleiben; sie werden nicht ungeprüft automatisch gelöscht.

## Eingabe- und Prozessgrenzen

- ExifTool-Benutzerkonfiguration und geerbte Perl-Optionen sind deaktiviert. Es gibt keine Shell-Befehlsverkettung, kein `eval`, keine unkontrollierten Tagselektoren und kein Frontend-Shell-Plugin.
- Metadaten werden als Text gerendert, niemals als HTML. Die CSP verhindert externe Inhalte und Verbindungen im Produktionsfrontend. Native Dateizugriffe beziehen sich auf ausdrücklich geöffnete Dateien; Scans folgen keinen Symlink-Unterbäumen.
- Mehrdeutige Tag-Instanzen bleiben lesbar, werden jedoch nicht auf einen einzelnen Schreibschlüssel reduziert. Der Writer prüft alle passenden Ergebnisinstanzen, nicht nur die erste.
- Ein Medienpreview darf fehlen. Decodierung ist auf 256 MiB geplante Decoder-Allokation, 30.000 Pixel pro Achse und eine gleichzeitige Anfrage begrenzt. ExifTool-Ausgabestreams sind auf je 32 MiB begrenzt. Das sind Schutzgrenzen, kein formaler Beweis gegen jeden Fehler in Drittanbieterdecodern.
- Die App besitzt keine Betriebssystem-Sandbox für die externen Decoder. Halten Sie ExifTool, Perl und die Bildbibliotheken aktuell. Unvertrauenswürdige, hochgradig adversarielle Dateien sollten zusätzlich in einer isolierten Umgebung analysiert werden.

## Bewusste Grenzen

SHA-256 und wiederholte Vergleiche erkennen externe Inhaltsänderungen auch bei gleichem Dateiumfang. Es gibt jedoch keine systemübergreifende exklusive Sperre gegen andere Programme: Zwischen letzter Prüfung und Austausch bleibt ein kleines Dateisystem-Rennfenster. Arbeiten in gleichzeitig schreibenden Programmen oder auf unzuverlässigen Netzwerkdateisystemen ist nicht als sicher verifiziert.

Die Backup-Garantie betrifft Dateiinhalt und normale Zugriffsrechte. Plattformübergreifende Erhaltung sämtlicher ACLs, Finder-Tags, Extended Attributes, alternativer NTFS-Datenströme, Birthtimes und Resource Forks ist noch nicht implementiert. Der Commit synchronisiert Dateiinhalte; vollständige Crash-Konsistenz der Verzeichniseinträge bei Stromausfall ist nicht nachgewiesen. Für wichtige Originale bleibt ein unabhängiges Backup erforderlich.

Der lokale SQLite-Cache und die Jobhistorie sind unverschlüsselt. Betriebssystemkontoschutz und Festplattenverschlüsselung liegen außerhalb der App. Sensible Werte können in Cache, Suchindex, Diffhistorie, Backups und Exporten verbleiben, auch wenn die veröffentlichte Mediendatei bereinigt wurde. Die App verspricht keine Bereinigung des gesamten Arbeitsplatzes. Eine automatische Retention-/Löschfunktion für Backups und Auditdaten ist nicht implementiert.

„Verwerfen“ betrifft nur ungespeicherte UI-Entwürfe; diese werden nicht als Wiederherstellungsmöglichkeit nach einem Absturz beworben. Die Anwendung fragt vor dem Schließen mit Entwürfen nach und verhindert reguläres Schließen während laufender Jobs. Erzwungenes Beenden kann trotzdem nur über Journal und Backup aufgefangen werden.

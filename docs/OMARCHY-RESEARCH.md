# Omarchy als Vorbild für den Open-Source-Start von Tagryn

Stand: 11. September 2026. Untersucht wurden die offiziellen Websites und das öffentliche Repository `omacom/omarchy` über Browser und GitHub CLI. Der untersuchte Default-Branch heißt `quattro`; sein Stand bei der Recherche war `b5589faaf80c6f87c07d4560fca37c4a81722f28`. Die Empfehlungen für Tagryn sind Vorschläge aus dem Vergleich, keine Aussagen über den noch separat zu prüfenden Tagryn-Code.

## Was Omarchy tatsächlich macht

### Produkt, Einstieg und Dokumentation

Omarchy präsentiert sich als Linux-Distribution mit einer klaren gestalterischen und technischen Handschrift. Die Website verbindet Produktversprechen, Videos, Installation, Dokumentation und Community-Beispiele. Sie bietet derzeit ein ISO für Version 4.0.3 samt SHA-256-Datei und Signatur sowie Links zu Möglichkeiten, Omarchy auf Mac und Windows in einer VM auszuprobieren. Das zeigt: Die öffentliche Präsentation führt direkt zu einem benutzbaren Ergebnis. [Offizielle Website](https://omarchy.org/)

Das README bleibt vergleichsweise knapp und verlinkt die im Repository gepflegte Benutzeranleitung. `manual/` ist laut README die maßgebliche Quelle; eine externe Website spiegelt diese Inhalte. Damit können Beiträge zu Code und Dokumentation gemeinsam versioniert werden. [README](https://github.com/omacom/omarchy/blob/b5589faaf80c6f87c07d4560fca37c4a81722f28/README.md)

### Lizenz und Marke

Das Repository steht unter MIT; die Lizenz nennt David Heinemeier Hansson als Copyright-Inhaber. MIT ist eine von der Open Source Initiative anerkannte Lizenz und erlaubt unter anderem Nutzung, Änderungen, Weiterverteilung und kommerzielle Verwertung, wenn der vorgeschriebene Copyright- und Lizenzhinweis erhalten bleibt. Für Tagryn bedeutet eine entsprechende Wahl daher auch, kommerzielle Weiterverwendung bewusst zuzulassen. [Omarchy-Lizenz](https://github.com/omacom/omarchy/blob/b5589faaf80c6f87c07d4560fca37c4a81722f28/LICENSE), [MIT bei der OSI](https://opensource.org/license/mit)

Omarchy behandelt die Marke gesondert: Die Markenseite bezeichnet Omarchy als angemeldete, noch nicht registrierte Marke und stellt Branding-Dateien bereit. Eine Open-Source-Code-Lizenz ersetzt also nicht automatisch eine bewusst formulierte Markenpolitik. [Markenseite](https://omarchy.org/brand/)

### KI-Beiträge sind ausdrücklich willkommen

Hier ist Omarchy ein direkt passendes Vorbild: Die Doctrine heißt Agenten ausdrücklich in Code, Issues, Pull Requests und Infrastruktur willkommen. Das ist eine belegte Projektposition und keine bloße Schlussfolgerung aus vorhandenen KI-Werkzeugen. [Doctrine, „Welcome the agents“](https://omarchy.org/doctrine/#welcome-the-agents)

Das Repository macht diese Haltung praktisch nutzbar. `AGENTS.md` beschreibt Konventionen, Zuständigkeiten von Verzeichnissen, Test-Einstiegspunkte und Anforderungen an visuelle Überprüfung. Aufgabenbezogene Anleitungen liegen unter `agents/skills/`, technische Referenz unter `docs/`, Benutzeranleitungen unter `manual/`. `CLAUDE.md` verweist auf `AGENTS.md`, statt eine zweite vollständige Regelsammlung zu pflegen. [AGENTS.md](https://github.com/omacom/omarchy/blob/b5589faaf80c6f87c07d4560fca37c4a81722f28/AGENTS.md), [CLAUDE.md](https://github.com/omacom/omarchy/blob/b5589faaf80c6f87c07d4560fca37c4a81722f28/CLAUDE.md)

In den geprüften Dateien und Seiten fand sich keine verbindliche Vorschrift, jede KI-Nutzung in einem PR offenzulegen. Eine solche Regel für Tagryn wäre eine eigene Entscheidung. Ebenso wurde im untersuchten Repository-Baum keine eigenständige `CONTRIBUTING.md` oder PR-Vorlage gefunden; GitHubs Community-Profil lieferte dafür ebenfalls keine Datei. Omarchy sollte deshalb als Inspiration dienen, nicht als vollständig zu kopierende Startvorlage. [Repository-Baum](https://github.com/omacom/omarchy/tree/b5589faaf80c6f87c07d4560fca37c4a81722f28), [GitHub-Community-Profil](https://api.github.com/repos/omacom/omarchy/community/profile)

### Führung und Beitragskanäle

Die Doctrine beschreibt eine klare, letztverantwortliche Projektführung mit Delegation, ausdrücklich keine Demokratie. Die Website nennt Teams für Core, Security, Design und Community-Hilfe. `CODEOWNERS` weist sämtliche Dateien `@dhh` und `@ryanrhughes` zu; der Kommentar verlangt deren Freigabe für Merges in geschützte Branches. Tatsächlich konfigurierte Branch-Schutzregeln wurden nicht separat verifiziert. [Doctrine, „Command is service“](https://omarchy.org/doctrine/#command-is-service), [Teams](https://omarchy.org/teams/), [CODEOWNERS](https://github.com/omacom/omarchy/blob/b5589faaf80c6f87c07d4560fca37c4a81722f28/.github/CODEOWNERS)

GitHub Issues sind für bestätigte Fehler vorgesehen. Das Fehlerformular verlangt Systeminformationen und eine Problembeschreibung. Ideen werden zu GitHub Discussions, Supportanfragen zu Discord gelenkt. Sicherheitsmeldungen sollen privat an das Security-Team gehen und ausreichend Informationen zur Reproduktion enthalten. [Fehlerformular](https://github.com/omacom/omarchy/blob/b5589faaf80c6f87c07d4560fca37c4a81722f28/.github/ISSUE_TEMPLATE/bug.yml), [Kanalkonfiguration](https://github.com/omacom/omarchy/blob/b5589faaf80c6f87c07d4560fca37c4a81722f28/.github/ISSUE_TEMPLATE/config.yml), [Security Policy](https://github.com/omacom/omarchy/blob/b5589faaf80c6f87c07d4560fca37c4a81722f28/.github/SECURITY.md)

### Releases, Updates und Tests

Die GitHub-Veröffentlichung v4.0.3 vom 8. September 2026 erklärt den Updateweg, verlinkt ISO und Prüfsumme, beschreibt Änderungen und nennt Beitragende. Die ISO liegt außerhalb von GitHub; der Release hatte bei der Abfrage keine hochgeladenen GitHub-Assets. [Release v4.0.3](https://github.com/omacom/omarchy/releases/tag/v4.0.3)

Omarchy dokumentiert Stable-, RC-, Edge- und Dev-Kanäle, reguläre Paketupdates, Migrationen und Wiederherstellung über Snapshots. Diese Komplexität passt zu einer Linux-Distribution; für Tagryns ersten Start muss sie nicht übernommen werden. [Update-Handbuch](https://github.com/omacom/omarchy/blob/b5589faaf80c6f87c07d4560fca37c4a81722f28/manual/30-updates.md)

Die Testdokumentation unterscheidet CLI-, Shell- und grafische Akzeptanztests. Grafische Tests laufen in einer Wegwerf-VM; der normale Sammelrunner umfasst diese nicht. In dem untersuchten Default-Branch enthielt `.github/` keine Workflow-Dateien. Daraus folgt keine Aussage über externe CI oder andere Repositories. [Testdokumentation](https://github.com/omacom/omarchy/blob/b5589faaf80c6f87c07d4560fca37c4a81722f28/docs/testing.md), [.github-Verzeichnis](https://github.com/omacom/omarchy/tree/b5589faaf80c6f87c07d4560fca37c4a81722f28/.github)

## Konkrete Vorschläge für Tagryn

Die folgenden Punkte sind eigene Empfehlungen für `franzgollhammer/tagryn`, abgeleitet aus den oben belegten Praktiken und dem gewünschten öffentlichen, KI-offenen Entwicklungsmodell:

1. **Ein benutzbares erstes Release zeigen.** README mit eindeutigem Nutzen, Screenshot oder kurzem Demo-Video, unterstützten Plattformen, reproduzierbarem Startweg und benannten Einschränkungen. Die tatsächlich unterstützte Installation muss anhand von Tagryn geprüft werden.
2. **Eine bewusste Open-Source-Lizenz setzen.** MIT entspricht Omarchys Modell, wenn weitgehend freie und kommerzielle Weiterverwendung gewollt ist. Vor öffentlicher Veröffentlichung müssen vorhandene Rechte und Lizenzen des konkreten Tagryn-Codes und seiner enthaltenen Assets geprüft werden. Ein öffentliches Repository allein ersetzt diese Lizenzentscheidung nicht.
3. **KI-Beiträge ausdrücklich einladen.** `CONTRIBUTING.md` sollte sagen, dass Beiträge mit oder ohne KI willkommen sind. Einreichende sollen das Problem erklären, den Diff prüfen, benötigte Nachweise liefern und auf Review reagieren können. Bewertung nach Qualität und nachvollziehbarem Verhalten, nicht nach verwendetem Werkzeug. Das ist der vorgeschlagene Tagryn-Standard, keine behauptete Omarchy-Regel.
4. **Eine knappe, gemeinsame Anleitung für Agenten pflegen.** `AGENTS.md` sollte echte Setup-Befehle, Architektur-Einstiegspunkte, wichtige Grenzen und den zulässigen Testweg enthalten. `CLAUDE.md` kann darauf verweisen. Persönliche Rechnerpfade und private Arbeitsabläufe gehören nicht in öffentliche Contributor-Anweisungen.
5. **Kleine Beiträge einfach machen.** Fehlerformular, PR-Vorlage mit Problem/Lösung/Validierung, einige klar begrenzte Einstiegsaufgaben und kurze Hinweise für Dokumentationsbeiträge. Für den Start reichen GitHub Issues und gegebenenfalls Discussions; ein zusätzlicher Chat ist eine optionale spätere Entscheidung.
6. **Verantwortung sichtbar halten.** Franz bleibt zunächst verantwortlicher Maintainer; Beiträge kommen über Forks und PRs. Dokumentieren, wer Richtung und Releases entscheidet. CODEOWNERS kann das unterstützen; tatsächliche Review-Regeln müssen zusätzlich in GitHub konfiguriert werden. Alle Merges bleiben an Franz’ bestehende Freigaberegel gebunden.
7. **Validierung und Release zusammenbringen.** Die vollständige Testsuite läuft nach Franz’ Vorgabe ausschließlich in GitHub Actions. Ein Release erhält Versionsnummer, verständliche Änderungen, bekannte Grenzen und genau den tatsächlich verfügbaren Installationsweg. Artefakte und Prüfsummen nur dort versprechen, wo die Build-Pipeline sie wirklich erzeugt.

Die entscheidende Übernahme ist die Kombination aus eigener Produktidee, öffentlicher Lizenz, leichtem Einstieg, klarer Maintainer-Verantwortung und expliziter Einladung an KI-gestützte Beiträge. Stiftung, Sponsorenprogramm und große Community-Struktur sind dafür keine Startvoraussetzung; Omarchy betreibt diese inzwischen als eigene Organisationsebene. [Omacom Foundation](https://omarchy.org/foundation/)

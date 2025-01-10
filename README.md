# GEMA Launcher

GEMA Launcher ist ein Hilfsprogramm zum Verarbeiten von Audiodateien und deren Metadaten, um GEMA-relevante Informationen effizient zusammenzustellen.

## Funktionen

### Dateiverwaltung
- Aus einer Liste von Audiodateien lassen sich einzelne oder alle Dateien entfernen.

### Parsing von Trackinformationen
- Die Anwendung extrahiert Titel, Künstler, Labelcodes und weitere Metadaten aus Dateinamen sowie begleitenden Dateien.

### Anreicherung durch Datenbanken
- **Labelcodes:** Eine JSON-Datenbank enthält zusätzliche Informationen zu Labelcodes. Diese werden bei Bedarf automatisch ergänzt und vereinheitlicht.
- **Künstler:** Eine SQLite-Datenbank (.db) speichert erweiterte Informationen über Künstler, die ebenfalls automatisch angereichert werden können.

### Formatierung der Ausgaben
- Die ermittelten und angereicherten Daten können in ein CSV-Format exportiert werden.
- Es wird sichergestellt, dass bestimmte Zeichenformate (z. B. Ersetzen von Kommas durch Unterstriche) konsequent angewandt werden, um eine saubere Datenstruktur zu gewährleisten.

### Einfache Erweiterbarkeit
- Der Code ist modular aufgebaut, sodass Datenquellen, Parsing-Logik und Ausgabemodi leicht angepasst oder erweitert werden können.

## Zusammenfassung

GEMA Launcher unterstützt dabei, aus einer Sammlung von Audiodateien schnell und effizient eine sauber aufbereitete CSV-Liste mit allen benötigten Musikmetadaten zu erstellen, um den GEMA-Meldeaufwand erheblich zu reduzieren.

**Full Changelog**: https://github.com/TJ-5/GEMA_RUST/commits/v.0.2.0

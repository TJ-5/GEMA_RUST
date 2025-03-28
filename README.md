# GEMA Launcher <img width="50" align="right" alt="logo" src="https://github.com/user-attachments/assets/fb76abad-1aa6-4409-b793-1525dac1d823" /> 

GEMA Launcher ist ein Hilfsprogramm zum Verarbeiten von Audiodateien und deren Metadaten, um GEMA-relevante Informationen effizient zusammenzustellen. 


<img width="934" alt="Screenshot 2025-01-16 at 11 47 30" src="https://github.com/user-attachments/assets/8b70f325-f844-425d-a837-2dc25f130198" />

## Installation
**Download**: https://github.com/TJ-5/GEMA_RUST/releases/tag/v3.0.12

Launcher ausführen und den Anweisungen folgen.

## Funktionen

### Parsing von Trackinformationen
- Die Anwendung extrahiert Titel, Künstler, Labelcodes und weitere Metadaten aus Dateinamen sowie begleitenden Dateien.

### Formatierung der Ausgaben
- Die ermittelten und angereicherten Daten können in ein CSV-Format exportiert werden.
- Es wird sichergestellt, dass bestimmte Zeichenformate (z. B. Ersetzen von Kommas durch Unterstriche) konsequent angewandt werden, um eine saubere Datenstruktur zu gewährleisten.

### Anreicherung durch Datenbanken
- **Labelcodes:** Eine JSON-Datenbank enthält zusätzliche Informationen zu Labelcodes. Diese werden bei Bedarf automatisch ergänzt und vereinheitlicht.
- **Künstler:** Die SQL-Datenbank von A3M wird benutzt um den Labelcode, die Künstler und den Titel zufinden

### Dateiverwaltung
- Aus einer Liste von Audiodateien lassen sich einzelne oder alle Dateien entfernen.

### Einfache Erweiterbarkeit
- Der Code ist modular aufgebaut, sodass Datenquellen, Parsing-Logik und Ausgabemodi leicht angepasst oder erweitert werden können.




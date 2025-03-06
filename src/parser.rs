use crate::model::TrackInfo;
use crate::app::GemaLauncherApp;
use anyhow::Result;
use log::{info, error};
use regex::Regex;
use std::fs::File; 
use std::io::{self, BufRead};
use std::path::Path;

/// Parst alle geladenen Dateien und füllt `tracks_per_file`.
pub fn parse_all_files(app: &mut GemaLauncherApp) -> Result<()> {
    app.tracks_per_file.clear();
    app.error_messages.clear();

    let re = Regex::new(r"^(?P<index>.*?_\d+_)(?P<titel>[A-Za-z_]+)_(?P<kuenstler>[^.]+)\.(wav|mp3)$").unwrap();

    // Kopie der Dateinamen, um Borrowing-Konflikte zu vermeiden
    let filenames_clone = app.filenames.clone();

    for filename in filenames_clone {
        let path = Path::new(&filename);

        // Falls es sich um eine .txt-Datei handelt, parse_text_file verwenden
        if path.extension().and_then(|s| s.to_str()) == Some("txt") {
            if let Err(e) = parse_text_file(app, &filename) {
                let msg = format!("Fehler beim Parsen der Textdatei {}: {}", filename, e);
                app.error_messages.push(msg.clone());
                error!("{}", msg);
            }
        } else {
            // Für nicht-TXT Dateien nutzen wir das bekannte Schema:
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
            if let Some(caps) = re.captures(&file_name) {
                let index = caps.name("index").map_or("", |m| m.as_str()).to_string();
                let titel = caps.name("titel").map_or("", |m| m.as_str()).to_string();
                let kuenstler = caps.name("kuenstler").map_or("", |m| m.as_str()).to_string();
                let label_code = find_label_code(&app.label_dict, &index);

                let track = TrackInfo {
                    index: index.clone(),
                    titel,
                    kuenstler,
                    duration: None,
                    label_code,
                };

                app.tracks_per_file.entry(filename.clone()).or_default().push(track);
                info!("Track extrahiert aus {}: {}", filename, file_name);
            } else {
                let msg = format!("Unbekanntes Format: {}", file_name);
                app.error_messages.push(msg.clone());
                error!("{}", msg);
            }
        }
    }

    Ok(())
}

/// Parst eine Textdatei im neuen Format:
///  - Erste Zeile IMMER überspringen.
///  - Pro Zeile: Alles bis zum ersten ':' ignorieren.
///  - Danach: Dauer-String bis zum nächsten '\t'.
///  - Danach: Dateiname/Trackinfo (Index, Titel, Künstler) wie gewohnt parsen.
fn parse_text_file(app: &mut GemaLauncherApp, path: &str) -> Result<()> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    if lines.len() <= 1 {
        // Falls die Datei leer oder nur eine Zeile hat, gibt es nichts zu parsen.
        app.error_messages.push(format!("Zu wenige Zeilen in '{}'", path));
        return Ok(());
    }

    // Erste Zeile immer überspringen
    for line in lines.iter().skip(1) {
        // Suchen nach erstem Doppelpunkt
        let Some(colon_pos) = line.find(':') else {
            // Kein Doppelpunkt => Zeile ignorieren oder als Fehler protokollieren
            let msg = format!("Kein ':' in Zeile, übersprungen: {}", line);
            app.error_messages.push(msg.clone());
            continue;
        };

        // Alles nach dem Doppelpunkt
        let after_colon = &line[colon_pos + 1..];

        // Dauer bis zum nächsten Tab auslesen (splitn(2, '\t') trennt nur 1x)
        let mut parts = after_colon.splitn(2, '\t');
        let duration_str = parts.next().unwrap_or("").trim();
        let track_str = parts.next().unwrap_or("").trim();

        if duration_str.is_empty() || track_str.is_empty() {
            let msg = format!("Zeile unvollständig (Dauer oder Track-Teil fehlt): {}", line);
            app.error_messages.push(msg.clone());
            continue;
        }

        let duration_in_seconds = parse_duration(duration_str);
        let Some(duration) = duration_in_seconds else {
            let msg = format!("Ungültige Dauer '{}' in Zeile '{}'", duration_str, line);
            app.error_messages.push(msg.clone());
            continue;
        };

        let (index, titel, kuenstler) = parse_track_filename(track_str);
        let label_code = find_label_code(&app.label_dict, &index);

        let tracks = app.tracks_per_file.entry(path.to_string()).or_default();
        if let Some(existing_track) = tracks.iter_mut().find(|t| {
            t.index == index && t.titel == titel && t.kuenstler == kuenstler
        }) {
            existing_track.duration = Some(existing_track.duration.unwrap_or(0.0) + duration);
            info!("Track aktualisiert (Dauer addiert): {} {} {}", index, titel, kuenstler);
        } else {
            tracks.push(TrackInfo {
                index,
                titel,
                kuenstler,
                duration: Some(duration),
                label_code,
            });
            info!("Neuer Track eingelesen: {}", track_str);
        }
    }
    Ok(())
}

/// Zerlegt den Filename in Index, Titel und Künstler
fn parse_track_filename(filename: &str) -> (String, String, String) {
    let original_base = filename.split('.').next().unwrap_or("");
    let base = original_base.replace('_', " ");
    let tokens = base.split_whitespace().collect::<Vec<&str>>();

    fn contains_digit(t: &str) -> bool {
        t.chars().any(|ch| ch.is_ascii_digit())
    }

    fn is_upper_token(t: &str) -> bool {
        t.chars().filter(|c| c.is_alphabetic()).all(|c| c.is_uppercase())
            && t.chars().any(|c| c.is_alphabetic())
    }

    let mut state = "BEFORE_DIGIT";
    let mut index_tokens = Vec::new();
    let mut title_tokens = Vec::new();
    let mut artist_tokens = Vec::new();

    for t in tokens {
        match state {
            "BEFORE_DIGIT" => {
                index_tokens.push(t);
                if contains_digit(t) {
                    state = "AFTER_DIGIT_BEFORE_TITLE";
                }
            }
            "AFTER_DIGIT_BEFORE_TITLE" => {
                if is_upper_token(t) {
                    title_tokens.push(t);
                    state = "TITLE";
                } else {
                    index_tokens.push(t);
                }
            }
            "TITLE" => {
                if is_upper_token(t) {
                    title_tokens.push(t);
                } else {
                    artist_tokens.push(t);
                    state = "ARTIST";
                }
            }
            "ARTIST" => {
                artist_tokens.push(t);
            }
            _ => {}
        }
    }

    let index_str = index_tokens.join("_").to_lowercase();
    let title_str = title_tokens.join(" ").to_lowercase();
    let artist_str = artist_tokens.join(" ").to_lowercase();

    (index_str, title_str, artist_str)
}

/// Konvertiert eine Zeitangabe (z. B. "1:23" oder "1.23") in Sekunden (f64).
fn parse_duration(duration_str: &str) -> Option<f64> {
    let duration_str = duration_str.replace(':', ".");
    let parts: Vec<&str> = duration_str.split('.').collect();

    if parts.len() < 2 {
        return None;
    }

    let main_part = parts[0];
    let decimal_part = parts[1];

    format!("{}.{}", main_part, decimal_part).parse::<f64>().ok()
}

/// Ermittelt anhand des Index (z.B. "JCM_123") den passenden Labelcode aus dem Dict.
fn find_label_code(label_dict: &std::collections::HashMap<String, String>, index_str: &str) -> String {
    for (label, code) in label_dict {
        if index_str.to_uppercase().starts_with(label) {
            return code.clone();
        }
    }
    String::new()
}

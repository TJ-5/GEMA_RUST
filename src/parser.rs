use crate::model::TrackInfo;
use crate::app::GemaLauncherApp;
use anyhow::Result;
use log::{info, error};
use regex::Regex;
use std::io::{self, BufRead};
use std::fs::File;
use std::path::Path;

/// Parst alle geladenen Dateien und füllt `tracks_per_file`.
pub fn parse_all_files(app: &mut GemaLauncherApp) -> Result<()> {
    app.tracks_per_file.clear();
    app.error_messages.clear();

    let re = Regex::new(r"^(?P<index>.*?)(?P<titel>[A-Z_]+)_(?P<kuenstler>[^.]+)\.(wav|mp3)$").unwrap();
    
    // Dateinamen klonen, um Borrowing-Konflikte zu vermeiden
    let filenames_clone = app.filenames.clone();

    for filename in filenames_clone {
        let path = Path::new(&filename);
        if path.extension().and_then(|s| s.to_str()) == Some("txt") {
            // Hier übergeben wir &mut app, aber benutzen filename aus der Kopie
            if let Err(e) = parse_text_file(app, &filename) {
                let msg = format!("Fehler beim Parsen der Textdatei {}: {}", filename, e);
                app.error_messages.push(msg.clone());
                error!("{}", msg);
            }
        } else {
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

                // Hier kann nun mutabel auf app zugegriffen werden, weil wir nicht mehr über &app.filenames iterieren
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

/// Parst eine Textdatei und extrahiert Tracks/Dauern.
fn parse_text_file(app: &mut GemaLauncherApp, path: &str) -> Result<()> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let lines = reader.lines().filter_map(Result::ok).collect::<Vec<String>>();

    if lines.is_empty() {
        app.error_messages.push(format!("Leere Textdatei: {}", path));
        return Ok(());
    }

    let is_alternating = lines.len() % 2 == 0 && lines.iter().enumerate().all(|(i, line)| {
        if i % 2 == 1 {
            line.contains(':')
        } else {
            true
        }
    });

    let mut track_duration_pairs = Vec::new();

    if is_alternating {
        for i in (0..lines.len()).step_by(2) {
            let track = lines[i].trim().to_string();
            let duration = lines[i + 1].trim().to_string();
            track_duration_pairs.push((track, duration));
        }
    } else {
        let half = lines.len() / 2;
        let tracks = &lines[..half];
        let durations = &lines[half..];

        if tracks.len() != durations.len() {
            app.error_messages.push(format!(
                "Die Anzahl der Tracks und Dauern stimmt nicht überein in Datei: {}",
                path
            ));
            return Ok(());
        }

        for (track, duration) in tracks.iter().zip(durations.iter()) {
            track_duration_pairs.push((track.trim().to_string(), duration.trim().to_string()));
        }
    }

    for (track, duration_str) in track_duration_pairs {
        let (index, titel, kuenstler) = parse_track_filename(&track);
        let duration_in_seconds = parse_duration(&duration_str);

        if duration_in_seconds.is_none() {
            let msg = format!("Ungültige Dauer '{}' für Track '{}'", duration_str, track);
            app.error_messages.push(msg.clone());
            error!("{}", msg);
            continue;
        }

        let duration = duration_in_seconds.unwrap();
        let label_code = find_label_code(&app.label_dict, &index);

        let tracks = app.tracks_per_file.entry(path.to_string()).or_default();
        if let Some(existing_track) = tracks.iter_mut().find(|t| t.index == index && t.titel == titel && t.kuenstler == kuenstler) {
            existing_track.duration = Some(existing_track.duration.unwrap_or(0.0) + duration);
            info!("Track aktualisiert: {} {} {}", index, titel, kuenstler);
        } else {
            tracks.push(TrackInfo {
                index,
                titel,
                kuenstler,
                duration: Some(duration),
                label_code,
            });
        }
    }
    Ok(())
}

/// Hilfsfunktionen
fn parse_track_filename(filename: &str) -> (String, String, String) {
    let original_base = filename.split('.').next().unwrap_or("");
    let base = original_base.replace('_', " ");
    let tokens = base.split_whitespace().collect::<Vec<&str>>();

    fn contains_digit(t: &str) -> bool {
        t.chars().any(|ch| ch.is_ascii_digit())
    }

    fn is_upper_token(t: &str) -> bool {
        t.chars().filter(|c| c.is_alphabetic()).all(|c| c.is_uppercase()) && t.chars().any(|c| c.is_alphabetic())
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

fn find_label_code(label_dict: &std::collections::HashMap<String, String>, index_str: &str) -> String {
    for (label, code) in label_dict {
        if index_str.to_uppercase().starts_with(label) {
            return code.clone();
        }
    }
    String::new()
}


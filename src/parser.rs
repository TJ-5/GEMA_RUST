use crate::model::TrackInfo;
use crate::app::GemaLauncherApp;
use anyhow::Result;
use log::{info, error};
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn parse_all_files(app: &mut GemaLauncherApp) -> Result<()> {
    app.tracks_per_file.clear();
    app.error_messages.clear();

    let filenames_clone = app.filenames.clone();

    for filename in filenames_clone {
        let path = Path::new(&filename);
        if path.extension().and_then(|s| s.to_str()) == Some("txt") {
            if let Err(e) = parse_text_file(app, &filename) {
                let msg = format!("Fehler beim Parsen der Textdatei {}: {}", filename, e);
                app.error_messages.push(msg.clone());
                error!("{}", msg);
            }
        } else {
            let msg = format!("Datei '{}' ist keine .txt und wird ignoriert.", filename);
            app.error_messages.push(msg.clone());
            error!("{}", msg);
        }
    }

    Ok(())
}

/// Beispielhaftes Einlesen deiner TXT-Datei:
fn parse_text_file(app: &mut GemaLauncherApp, path: &str) -> Result<()> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let mut lines_iter = reader.lines();
    // 1. Zeile (Header) überspringen
    lines_iter.next();

    for line_result in lines_iter {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.trim().is_empty() {
            continue;
        }

        // Zerlegen nach Tabs
        let columns: Vec<&str> = line.split('\t').map(|s| s.trim()).collect();
        if columns.len() < 3 {
            let msg = format!("Unerwartetes Zeilenformat (weniger als 3 Spalten): {}", line);
            app.error_messages.push(msg.clone());
            continue;
        }

        let duration_str = columns[1];
        let track_str = columns[2];

        let Some(duration_in_seconds) = parse_hh_mm_ss_frames(duration_str) else {
            let msg = format!("Konnte Dauer '{}' nicht parsen: {}", duration_str, line);
            app.error_messages.push(msg.clone());
            continue;
        };

        // Aus dem Dateinamen index/titel/kuenstler holen
        let (index, titel, kuenstler) = parse_track_filename(track_str);

        let label_code = find_label_code(&app.label_dict, &index);

        let tracks = app.tracks_per_file.entry(path.to_string()).or_default();
        if let Some(existing_track) = tracks.iter_mut().find(|t| {
            t.index == index && t.titel == titel && t.kuenstler == kuenstler
        }) {
            existing_track.duration = Some(existing_track.duration.unwrap_or(0.0) + duration_in_seconds);
            info!("Track aktualisiert (Dauer addiert): {} {} {}", index, titel, kuenstler);
        } else {
            // Erst loggen, dann verschieben (um Move-Fehler zu vermeiden)
            info!("Neuer Track geparst: {} {} {}", index, titel, kuenstler);

            tracks.push(TrackInfo {
                index,
                titel,
                kuenstler,
                duration: Some(duration_in_seconds),
                label_code,
            });
        }
    }

    Ok(())
}

/// Parsen von "ANW1832_001_Forgotten-Dreams.wav.new.01"
///  1) cut alles hinter .wav/.mp3
///  2) cut .wav/.mp3 selbst weg
///  3) split_index_and_rest => (index_part, rest_part)
///  4) Letzten Unterstrich vom index weg, falls vorhanden
///  5) Titel + Künstler aufsplitten
fn parse_track_filename(filename: &str) -> (String, String, String) {
    // 1) Alles hinter .wav / .mp3 weg
    let base_with_ext = strip_version(filename);

    // 2) .wav / .mp3 entfernen
    let base_no_ext = match base_with_ext.rsplit_once('.') {
        Some((b, _ext)) => b,
        None => base_with_ext,
    };

    // 3) Bis zu 2 "_123_"-Blöcke rausholen
    let (mut index_part, rest_part) = match split_index_and_rest(base_no_ext) {
        Some(t) => t,
        None => (base_no_ext.to_string(), "".to_string()),
    };

    // 4) Wenn index_part mit '_' endet, abschneiden
    if index_part.ends_with('_') {
        index_part.pop(); // entfernt das letzte Zeichen
    }

    // 5) rest_part in titel + kuenstler zerlegen
    let (titel, kuenstler) = split_title_and_artist(rest_part);

    // in Kleinschreibung
    (
        index_part.to_lowercase(),
        titel.to_lowercase(),
        kuenstler.to_lowercase(),
    )
}

/// Schneidet alles nach ".wav" oder ".mp3" ab, z.B. ".wav.new.01"
fn strip_version(filename: &str) -> &str {
    let lower = filename.to_lowercase();
    if let Some(pos) = lower.rfind(".mp3") {
        &filename[..pos + 4]
    } else if let Some(pos) = lower.rfind(".wav") {
        &filename[..pos + 4]
    } else {
        filename
    }
}

/// Regex, um bis zu 2 `_Ziffern_`-Blöcke als "index" zu erkennen
fn split_index_and_rest(base: &str) -> Option<(String, String)> {
    let re = Regex::new(r"^(?P<index>.*?_\d+_(?:\d+_)?)(?P<rest>.*)$").unwrap();
    re.captures(base).map(|caps| {
        let index_str = caps["index"].to_string();
        let rest_str  = caps["rest"].to_string();
        (index_str, rest_str)
    })
}

/// Wir teilen "Heroes_Remembered___Beck_Gilmartin" am letzten Unterstrich
fn split_title_and_artist(rest: String) -> (String, String) {
    if rest.is_empty() {
        return ("".to_string(), "".to_string());
    }
    let tokens = rest.split('_').collect::<Vec<_>>();
    if tokens.len() == 1 {
        (tokens[0].to_string(), "".to_string())
    } else {
        let title = tokens[..(tokens.len() - 1)].join("_");
        let artist = tokens[tokens.len() - 1];
        (title, artist.to_string())
    }
}

/// Parst die Dauer "00:00:43:12" (HH:MM:SS:Frames, 25 fps als Beispiel)
fn parse_hh_mm_ss_frames(dur: &str) -> Option<f64> {
    let parts: Vec<&str> = dur.split(':').collect();
    if parts.len() != 4 {
        return None;
    }
    let hh = parts[0].parse::<u64>().ok()?;
    let mm = parts[1].parse::<u64>().ok()?;
    let ss = parts[2].parse::<u64>().ok()?;
    let frames = parts[3].parse::<u64>().ok()?;

    let fps = 25.0;
    let total_seconds = (hh * 3600 + mm * 60 + ss) as f64 + frames as f64 / fps;
    Some(total_seconds)
}

/// Liest label_code basierend auf dem index-Str (z.B. "ANW", "BMGPM", etc.)
fn find_label_code(label_dict: &std::collections::HashMap<String, String>, index_str: &str) -> String {
    for (label, code) in label_dict {
        if index_str.to_uppercase().starts_with(&label.to_uppercase()) {
            return code.clone();
        }
    }
    String::new()
}
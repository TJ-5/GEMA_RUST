// src/export.rs
use crate::app::GemaLauncherApp;
use anyhow::Result;
use log::{error, info};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use rfd::FileDialog;

impl GemaLauncherApp {
    pub fn export_all_csv(&mut self) -> Result<()> {
        if self.tracks_per_file.is_empty() {
            rfd::MessageDialog::new()
                .set_title("Keine Daten")
                .set_description("Es gibt keine Daten zum Exportieren.")
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
            return Ok(());
        }

        // Falls kein Pfad gesetzt ist, frage den Nutzer nach einem Ordner
        let export_dir: PathBuf = if let Some(path) = &self.export_path {
            PathBuf::from(path)
        } else {
            // Ordner wählen, wenn keiner gesetzt ist
            if let Some(folder) = FileDialog::new().pick_folder() {
                folder
            } else {
                // User hat abgebrochen
                return Ok(());
            }
        };

        for (filename, tracks) in &self.tracks_per_file {
            if tracks.is_empty() {
                continue;
            }

            let path = Path::new(filename);
            let base_name = path.file_stem().unwrap_or_default().to_str().unwrap_or("output");
            let formatted_name = format!("{}_formatted.csv", base_name);
            let output_path = export_dir.join(&formatted_name);

            match File::create(&output_path) {
                Ok(mut f) => {
                    // CSV Header
                    if let Err(e) = writeln!(f, "Index,Titel,Künstler,Dauer,Labelcode") {
                        let error_msg = format!("CSV-Fehler: {}", e);
                        self.error_messages.push(error_msg.clone());
                        error!("{}", error_msg);
                        continue;
                    }

                    // CSV Daten
                    for track in tracks {
                        let duration = track.duration.map_or(String::new(), |d| self.format_duration(d));
                        if let Err(e) = writeln!(f, "{},{},{},{},{}",
                            track.index,
                            track.titel,
                            track.kuenstler,
                            duration,
                            track.label_code) {
                            let error_msg = format!("CSV-Fehler: {}", e);
                            self.error_messages.push(error_msg.clone());
                            error!("{}", error_msg);
                            continue;
                        }
                    }

                    info!("CSV erfolgreich exportiert nach {}", output_path.display());
                }
                Err(e) => {
                    let error_msg = format!("Datei-Fehler: {}", e);
                    self.error_messages.push(error_msg.clone());
                    error!("{}", error_msg);
                }
            }
        }

        rfd::MessageDialog::new()
            .set_title("Erfolg")
            .set_description("Alle CSVs wurden erfolgreich exportiert.")
            .set_buttons(rfd::MessageButtons::Ok)
            .show();

        Ok(())
    }
}
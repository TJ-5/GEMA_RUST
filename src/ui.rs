// src/ui.rs
use eframe::egui;
use crate::app::GemaLauncherApp;
use log::info;
use rfd::FileDialog;
use eframe::App;


impl App for GemaLauncherApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let dropped_files = ctx.input(|input| input.raw.dropped_files.clone());

        if !dropped_files.is_empty() {
            for file in dropped_files.iter() {
                if let Some(path_str) = file.path.as_ref().and_then(|p| p.to_str()) {
                    info!("Datei per Drag-and-Drop hinzugefügt: {}", path_str);
                    self.add_file(path_str.to_string());
                }
            }
            let _ = self.parse_filenames();
        }

        // Menüleiste oben
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Datei", |ui| {
                    if ui.button("Exportieren").clicked() {
                        let _ = self.export_all_csv();
                    }

                    if ui.button("Beenden").clicked() { 
                        frame.close()
                    }
                });

                ui.menu_button("Hilfe", |ui| {
                    ui.label("Tom fragen :)");
                });
            });
        });

        // Seitenleiste für Aktionen
        egui::SidePanel::left("side_panel").resizable(true).show(ctx, |ui| {
            ui.add_space(5.0);
            ui.heading("Aktionen");
            ui.add_space(5.0);

            if ui.button("Dateien auswählen").clicked() {
                if let Some(files) = FileDialog::new()
                    .add_filter("Audio/Text Dateien", &["wav", "mp3", "txt"])
                    .pick_files()
                {
                    for file in files {
                        if let Some(path_str) = file.to_str() {
                            self.add_file(path_str.to_string());
                        }
                    }
                    let _ = self.parse_filenames();
                }
            }
            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);

            if ui.button("Ausgewählte löschen").clicked() {
                self.delete_selected_files();
            }

            ui.add_space(5.0);

            if ui.button("Alle löschen").clicked() {
                self.delete_all_files();
            }

            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);

            if ui.button("Export Pfad wählen").clicked() {
                if let Some(folder) = FileDialog::new().pick_folder() {
                    self.export_path = Some(folder.to_string_lossy().to_string());
                }
            }

            if let Some(path) = &self.export_path {
                ui.label(format!("Export Pfad: {}", path));
            } else {
                ui.add_space(5.0);
                ui.label("Kein Exportpfad gewählt");
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            if ui.button("Export").clicked() {
                let _ = self.export_all_csv();
            }

        });

        // Hauptbereich
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("GEMA_Launcher - Dateinamen Parser zu CSV");
            ui.add_space(10.0);

            ui.collapsing("Geladene Dateien", |ui| {
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for (i, filename) in self.filenames.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.selected_files[i], "");
                            ui.label(filename);
                        });
                    }
                });
            });

            ui.separator();

            // Tracks zählen
            let total_tracks: usize = self.tracks_per_file.values().map(|v| v.len()).sum();
            ui.heading(format!("Extrahierte Tracks: {}", total_tracks));

            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                for (file, tracks) in &self.tracks_per_file {
                    ui.group(|ui| {
                        ui.label(format!("Datei: {}", file));
                        ui.separator();
                        for track in tracks {
                            ui.horizontal(|ui| {
                                ui.label(&track.index);
                                ui.label(&track.titel);
                                ui.label(&track.kuenstler);
                                if let Some(dauer) = track.duration {
                                    ui.label(format!("Dauer: {:.2} Sekunden", dauer));
                                }
                                ui.label(&track.label_code);
                            });
                        }
                    });
                    ui.separator();
                }
            });

            if !self.error_messages.is_empty() {
                ui.add_space(20.0);
                ui.separator();
                ui.colored_label(egui::Color32::RED, "Fehlerhafte Einträge:");
                egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                    for error in &self.error_messages {
                        ui.label(error);
                    }
                });
            }
        });
    }
}
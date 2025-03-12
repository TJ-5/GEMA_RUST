use eframe::egui;
use std::process::Command;
use crate::app::GemaLauncherApp;
use log::info;
use rfd::FileDialog;
use eframe::App;
use webbrowser;
use rusqlite::params;

impl App for GemaLauncherApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            let _ = self.export_all_csv();
        }
        
        // Process drag and drop
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

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Datei", |ui| {
                    if ui.button("Exportieren").clicked() {
                        let _ = self.export_all_csv();
                    }
                    ui.menu_button("Datenbank", |ui| {
                        // Bestehende Buttons
                        if ui.button("Datensatz aktualisieren").clicked() {
                            self.show_db_update_dialog = true;
                        }
                        if ui.button("Datenbank durchsuchen").clicked() {
                            self.show_db_search_dialog = true;
                            self.db_search_results.clear();
                        }
                        
                        // HIER DIE NEUEN EINTRÄGE EINFÜGEN:
                        ui.separator();
                        if ui.button("Datenbank optimieren").clicked() {
                            if let Err(e) = self.analyze_database() {
                                self.error_messages.push(format!("Fehler bei Datenbankoptimierung: {}", e));
                            } else {
                                rfd::MessageDialog::new()
                                    .set_title("Erfolg")
                                    .set_description("Datenbank wurde erfolgreich optimiert.")
                                    .set_buttons(rfd::MessageButtons::Ok)
                                    .show();
                            }
                        }
                        
                        if ui.button("Datenbank komprimieren").clicked() {
                            if let Err(e) = self.vacuum_database() {
                                self.error_messages.push(format!("Fehler bei Datenbankkomprimierung: {}", e));
                            } else {
                                rfd::MessageDialog::new()
                                    .set_title("Erfolg")
                                    .set_description("Datenbank wurde erfolgreich komprimiert.")
                                    .set_buttons(rfd::MessageButtons::Ok)
                                    .show();
                            }
                        }
                    });

                    if ui.button("CSV Vorschau").clicked() {
                        self.show_csv_preview = !self.show_csv_preview;
                        if self.show_csv_preview && !self.filenames.is_empty() {
                            self.selected_csv_file = Some(self.filenames[0].clone());
                        }
                    }

                    ui.separator();
                    if ui.button("Beenden").clicked() {
                        frame.close();
                    }
                });

                ui.menu_button("Datenbank", |ui| {
                    if ui.button("Datensatz aktualisieren").clicked() {
                        self.show_db_update_dialog = true;
                    }
                    if ui.button("Datenbank durchsuchen").clicked() {
                        self.show_db_search_dialog = true;
                        self.db_search_results.clear();
                    }
                });

                ui.menu_button("Hilfe", |ui| {
                    if ui.button("Welcome").clicked() {
                        let _ = webbrowser::open("https://github.com/TJ-5/GEMA_RUST/blob/main/README.pdf");
                    }
                    if ui.button("Mail").clicked() {
                        #[cfg(target_os = "windows")]
                        let _ = Command::new("cmd")
                            .args(&["/C", "start", "mailto:tom.joeres@filmpool.de?subject=Hilfe&body=Hallo%20Tom"])
                            .spawn();
                
                        #[cfg(target_os = "macos")]
                        let _ = Command::new("open")
                            .arg("mailto:tom.joeres@filmpool.de?subject=Hilfe&body=Hallo%20Tom")
                            .spawn();
                
                        #[cfg(target_os = "linux")]
                        let _ = Command::new("xdg-email")
                            .args(&["--subject", "Hilfe", "--body", "Hallo Tom,", "tom@example.com"])
                            .spawn();
                    }
                });
            });
        });

        // Left side panel for actions
        egui::SidePanel::left("side_panel").resizable(true).min_width(200.0).show(ctx, |ui| {
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
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            
            // Quick search for tracks
            ui.heading("Track Suche");
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.label("Suchbegriff:");
                ui.text_edit_singleline(&mut self.track_search_query);
            });
            
            if !self.track_search_query.is_empty() {
                ui.add_space(5.0);
                ui.label("Suchergebnisse:");
                let query = self.track_search_query.to_lowercase();
                
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    let mut found = false;
                    
                    for (_file, tracks) in &self.tracks_per_file {
                        for track in tracks {
                            if track.index.to_lowercase().contains(&query) || 
                               track.titel.to_lowercase().contains(&query) || 
                               track.kuenstler.to_lowercase().contains(&query) {
                                found = true;
                                
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.strong(&track.index);
                                    });
                                    ui.label(&track.titel);
                                    ui.label(&track.kuenstler);
                                    if let Some(duration) = track.duration {
                                        ui.label(format!("{}", self.format_duration(duration)));
                                    }
                                    ui.label(&track.label_code);
                                });
                            }
                        }
                    }
                    
                    if !found {
                        ui.label("Keine Ergebnisse gefunden");
                    }
                });
            }
        });

        // Main area with tabs
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("GEMA_Launcher - Dateinamen Parser zu CSV");
            ui.add_space(10.0);

            // Main tabs
            egui::TopBottomPanel::top("tabs").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.show_csv_preview, false, "Track Übersicht");
                    ui.selectable_value(&mut self.show_csv_preview, true, "CSV Vorschau");
                });
            });

            // Content based on selected tab
            if self.show_csv_preview {
                self.render_csv_preview(ui);
            } else {
                self.render_tracks_view(ui);
            }

            // Error messages
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

        // Database update dialog
        if self.show_db_update_dialog {
            self.render_db_update_dialog(ctx);
        }

        // Database search dialog
        if self.show_db_search_dialog {
            self.render_db_search_dialog(ctx);
        }
    }
}

// UI rendering methods implementation
impl GemaLauncherApp {
    // Render the tracks overview
    fn render_tracks_view(&self, ui: &mut egui::Ui) {
        ui.collapsing("Geladene Dateien", |ui| {
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                for (i, filename) in self.filenames.iter().enumerate() {
                    ui.horizontal(|ui| {
                        if i < self.selected_files.len() {
                            let mut selected = self.selected_files[i];
                            if ui.checkbox(&mut selected, "").changed() {
                                // Since we can't modify self directly here, we just display
                                // The actual modification would need to be handled elsewhere
                            }
                            ui.label(filename);
                        }
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
                    
                    // Create a grid for tabular layout
                    egui::Grid::new(format!("tracks_grid_{}", file))
                        .num_columns(5)
                        .spacing([8.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // Header row
                            ui.strong("Index");
                            ui.strong("Titel");
                            ui.strong("Künstler");
                            ui.strong("Dauer");
                            ui.strong("Labelcode");
                            ui.end_row();
                            
                            // Data rows
                            for track in tracks {
                                ui.label(&track.index);
                                ui.label(&track.titel);
                                ui.label(&track.kuenstler);
                                if let Some(dauer) = track.duration {
                                    ui.label(self.format_duration(dauer));
                                } else {
                                    ui.label("-");
                                }
                                ui.label(&track.label_code);
                                ui.end_row();
                            }
                        });
                });
                ui.separator();
            }
        });
    }

    // Render CSV preview
    fn render_csv_preview(&mut self, ui: &mut egui::Ui) {
        ui.heading("CSV Vorschau");
        ui.add_space(10.0);
        
        // Dropdown to select file for preview
        ui.horizontal(|ui| {
            ui.label("Datei auswählen:");
            egui::ComboBox::from_id_source("csv_file_selector")
                .selected_text(self.selected_csv_file.as_deref().unwrap_or("Keine Datei ausgewählt"))
                .show_ui(ui, |ui| {
                    for filename in &self.filenames {
                        let selected = Some(filename.clone()) == self.selected_csv_file;
                        if ui.selectable_label(selected, filename).clicked() {
                            self.selected_csv_file = Some(filename.clone());
                        }
                    }
                });
        });
        
        ui.add_space(10.0);
        
        // CSV table preview
        if let Some(selected_file) = &self.selected_csv_file {
            if let Some(tracks) = self.tracks_per_file.get(selected_file) {
                ui.group(|ui| {
                    // Add a scroll area for both horizontal and vertical scrolling
                    egui::ScrollArea::both()
                        .max_height(400.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            // Force a minimum width to ensure horizontal scrolling is available
                            let available_width = ui.available_width();
                            let min_content_width = available_width.max(800.0); // At least 800px wide or screen width
                            
                            egui::Frame::none()
                                //.fill(egui::Color32::from_rgb(102, 102, 102))
                                .show(ui, |ui| {
                                    ui.allocate_space(egui::Vec2::new(min_content_width, 0.0));
                                    
                                    // Use Grid for tabular layout
                                    egui::Grid::new("csv_preview_grid")
                                        .num_columns(5)
                                        .spacing([20.0, 6.0]) // More spacing between columns
                                        .striped(true)
                                        .show(ui, |ui| {
                                            // Header row with extra spacing
                                            ui.strong("Index");
                                            ui.strong("Titel");
                                            ui.strong("Künstler");
                                            ui.strong("Dauer");
                                            ui.strong("Labelcode");
                                            ui.end_row();
                                            
                                            // Data rows - ensure long content doesn't get truncated
                                            for track in tracks {
                                                ui.label(&track.index);
                                                ui.label(&track.titel);
                                                ui.label(&track.kuenstler);
                                                if let Some(dauer) = track.duration {
                                                    ui.label(format!("' {}", self.format_duration(dauer)));
                                                } else {
                                                    ui.label("-");
                                                }
                                                ui.label(&track.label_code);
                                                ui.end_row();
                                            }
                                        });
                                });
                        });
                });
            } else {
                ui.label("Keine Daten für diese Datei gefunden.");
            }
        } else {
            ui.label("Bitte wählen Sie eine Datei aus, um die Vorschau anzuzeigen.");
        }
    }

    // Render database update dialog
    fn render_db_update_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Datenbank aktualisieren")
            .resizable(true)
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Track-Daten in Datenbank aktualisieren");
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label("Index:");
                    ui.text_edit_singleline(&mut self.db_update_index);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Titel:");
                    ui.text_edit_singleline(&mut self.db_update_title);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Künstler:");
                    ui.text_edit_singleline(&mut self.db_update_artist);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Labelcode:");
                    ui.text_edit_singleline(&mut self.db_update_labelcode);
                });
                
                ui.add_space(10.0);
                
                // Status message
                if !self.db_update_status.is_empty() {
                    ui.colored_label(
                        if self.db_update_status.contains("Fehler") {
                            egui::Color32::RED
                        } else {
                            egui::Color32::GREEN
                        },
                        &self.db_update_status
                    );
                }
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Speichern").clicked() {
                        self.update_database_entry();
                    }
                    
                    if ui.button("Schließen").clicked() {
                        self.show_db_update_dialog = false;
                    }
                });
            });
    }
    
    // Method to update database
    fn update_database_entry(&mut self) {
        if self.db_update_index.is_empty() {
            self.db_update_status = "Fehler: Index ist erforderlich".to_string();
            return;
        }
        
        let Some(conn) = self.db_connection.as_ref() else {
            self.db_update_status = "Fehler: Keine Datenbankverbindung".to_string();
            return;
        };
        
        // Check if entry exists
        let exists_query = "SELECT COUNT(*) FROM my_table WHERE LOWER(\"index\") = LOWER(?1)";
        let exists: i64 = match conn.query_row(exists_query, params![&self.db_update_index], |row| row.get(0)) {
            Ok(count) => count,
            Err(e) => {
                self.db_update_status = format!("Fehler bei Datenbankabfrage: {}", e);
                return;
            }
        };
        
        let result = if exists > 0 {
            // Update existing entry
            conn.execute(
                "UPDATE my_table SET titel = ?1, kuenstler = ?2, labelcode = ?3 WHERE LOWER(\"index\") = LOWER(?4)",
                params![
                    &self.db_update_title,
                    &self.db_update_artist,
                    &self.db_update_labelcode,
                    &self.db_update_index,
                ],
            )
        } else {
            // Insert new entry
            conn.execute(
                "INSERT INTO my_table (\"index\", titel, kuenstler, labelcode) VALUES (?1, ?2, ?3, ?4)",
                params![
                    &self.db_update_index,
                    &self.db_update_title,
                    &self.db_update_artist,
                    &self.db_update_labelcode,
                ],
            )
        };
        
        match result {
            Ok(_) => {
                self.db_update_status = format!("Eintrag erfolgreich {}", if exists > 0 { "aktualisiert" } else { "hinzugefügt" });
                // Re-parse to apply database changes
                let _ = self.parse_filenames();
            },
            Err(e) => {
                self.db_update_status = format!("Datenbankfehler: {}", e);
            }
        }
    }

    // Render database search dialog
    fn render_db_search_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Datenbank durchsuchen")
            .resizable(true)
            .min_width(550.0)
            .min_height(400.0)
            .show(ctx, |ui| {
                ui.heading("Datenbank durchsuchen");
                ui.add_space(10.0);
                
                // Search options
                ui.horizontal(|ui| {
                    ui.label("Suchen in:");
                    ui.checkbox(&mut self.db_search_in_index, "Index");
                    ui.checkbox(&mut self.db_search_in_title, "Titel");
                    ui.checkbox(&mut self.db_search_in_artist, "Künstler");
                    ui.checkbox(&mut self.db_search_in_labelcode, "Labelcode");
                });
                
                ui.add_space(5.0);
                
                // Search input
                ui.horizontal(|ui| {
                    ui.label("Suchbegriff:");
                    let text_response = ui.text_edit_singleline(&mut self.db_search_query);
                    
                    if text_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) || 
                       ui.button("Suchen").clicked() {
                        self.perform_database_search();
                    }
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                // Search results table
                ui.heading(format!("Ergebnisse ({})", self.db_search_results.len()));
                ui.add_space(5.0);
                
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    // Use Grid for tabular layout
                    egui::Grid::new("db_search_results_grid")
                        .num_columns(5) // 4 columns + 1 for the edit button
                        .spacing([10.0, 6.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // Header row
                            ui.strong("Index");
                            ui.strong("Titel");
                            ui.strong("Künstler");
                            ui.strong("Labelcode");
                            ui.strong(""); // Empty header for the button column
                            ui.end_row();
                            
                            // Table rows
                            for (index, title, artist, labelcode) in &self.db_search_results {
                                ui.label(index);
                                ui.label(title);
                                ui.label(artist);
                                ui.label(labelcode);
                                
                                // Button to edit this entry
                                if ui.button("Bearbeiten").clicked() {
                                    self.db_update_index = index.clone();
                                    self.db_update_title = title.clone();
                                    self.db_update_artist = artist.clone();
                                    self.db_update_labelcode = labelcode.clone();
                                    self.show_db_update_dialog = true;
                                    self.show_db_search_dialog = false;
                                }
                                ui.end_row();
                            }
                        });
                    
                    if self.db_search_results.is_empty() && !self.db_search_query.is_empty() {
                        ui.label("Keine Ergebnisse gefunden.");
                    }
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Neuen Eintrag erstellen").clicked() {
                        // Preset search query as index if available
                        self.db_update_index = self.db_search_query.clone();
                        self.db_update_title = String::new();
                        self.db_update_artist = String::new();
                        self.db_update_labelcode = String::new();
                        self.show_db_update_dialog = true;
                        self.show_db_search_dialog = false;
                    }
                    
                    if ui.button("Schließen").clicked() {
                        self.show_db_search_dialog = false;
                    }
                });
            });
    }
    
    // Method to perform database search
    fn perform_database_search(&mut self) {
        if self.db_search_query.is_empty() {
            return;
        }
        
        let Some(conn) = self.db_connection.as_ref() else {
            return;
        };
        
        // Clear previous results
        self.db_search_results.clear();
        
        // Build the query based on search options
        let mut conditions = Vec::new();
        let search_pattern = format!("%{}%", self.db_search_query.to_lowercase());
        
        if self.db_search_in_index {
            conditions.push("LOWER(\"index\") LIKE LOWER(?1)".to_string());
        }
        
        if self.db_search_in_title {
            conditions.push("LOWER(titel) LIKE LOWER(?1)".to_string());
        }
        
        if self.db_search_in_artist {
            conditions.push("LOWER(kuenstler) LIKE LOWER(?1)".to_string());
        }
        
        if self.db_search_in_labelcode {
            conditions.push("LOWER(labelcode) LIKE LOWER(?1)".to_string());
        }
        
        if conditions.is_empty() {
            // At least one search field must be selected
            return;
        }
        
        let query = format!(
            "SELECT \"index\", titel, kuenstler, labelcode FROM my_table WHERE {} LIMIT 100",
            conditions.join(" OR ")
        );
        
        let mut stmt = match conn.prepare(&query) {
            Ok(stmt) => stmt,
            Err(_) => return,
        };
        
        let rows = match stmt.query_map(params![search_pattern], |row| {
            Ok((
                row.get::<_, String>(0)?, // index
                row.get::<_, String>(1)?, // titel
                row.get::<_, String>(2)?, // kuenstler
                row.get::<_, String>(3)?, // labelcode
            ))
        }) {
            Ok(rows) => rows,
            Err(_) => return,
        };
        
        for row_result in rows {
            if let Ok(row) = row_result {
                self.db_search_results.push(row);
            }
        }
    }
}
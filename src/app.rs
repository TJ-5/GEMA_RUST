use rusqlite::{params, Connection};
use anyhow::Result;
use log::info;
use std::collections::HashMap;
use crate::labelcodes::load_labelcodes;
use crate::model::TrackInfo;


pub struct GemaLauncherApp {
    // Bestehende Felder
    pub filenames: Vec<String>,
    pub error_messages: Vec<String>,
    pub label_dict: HashMap<String, String>,
    pub selected_files: Vec<bool>,
    pub tracks_per_file: HashMap<String, Vec<TrackInfo>>,
    pub export_path: Option<String>,
    pub db_connection: Option<Connection>,
    
    // Neue Felder für UI
    pub show_csv_preview: bool,
    pub selected_csv_file: Option<String>,
    pub show_db_update_dialog: bool,
    pub show_db_search_dialog: bool,
    pub track_search_query: String,
    
    // Felder für Datenbank-Aktualisierung
    pub db_update_index: String,
    pub db_update_title: String,
    pub db_update_artist: String,
    pub db_update_labelcode: String,
    pub db_update_status: String,
    
    // Felder für Datenbank-Suche
    pub db_search_query: String,
    pub db_search_results: Vec<(String, String, String, String)>, // (index, title, artist, labelcode)
    pub db_search_in_index: bool,
    pub db_search_in_title: bool,
    pub db_search_in_artist: bool,
    pub db_search_in_labelcode: bool,
}

// In der Default-Implementation diese Felder initialisieren
impl Default for GemaLauncherApp {
    fn default() -> Self {
        let mut app = Self {
            // Bestehende Felder
            filenames: Vec::new(),
            error_messages: Vec::new(),
            label_dict: load_labelcodes("src/assets/labelcodes.json").unwrap_or_default(),
            selected_files: Vec::new(),
            tracks_per_file: HashMap::new(),
            export_path: None,
            db_connection: None,
            
            // Neue Felder initialisieren
            show_csv_preview: false,
            selected_csv_file: None,
            show_db_update_dialog: false,
            show_db_search_dialog: false,
            track_search_query: String::new(),
            
            db_update_index: String::new(),
            db_update_title: String::new(),
            db_update_artist: String::new(),
            db_update_labelcode: String::new(),
            db_update_status: String::new(),
            
            db_search_query: String::new(),
            db_search_results: Vec::new(),
            db_search_in_index: true,
            db_search_in_title: true,
            db_search_in_artist: true,
            db_search_in_labelcode: true,
        };

        if let Err(e) = app.connect_to_database("src/assets/databank.db") {
            app.error_messages.push(format!("Datenbank konnte nicht geladen werden: {}", e));
        }
        app
    }
}

impl GemaLauncherApp {
    fn connect_to_database(&mut self, path: &str) -> Result<()> {
        let conn = Connection::open(path)?;
        
        // SQLite-Optimierungen direkt anwenden
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = 10000;
            PRAGMA temp_store = MEMORY;
        ")?;
        
        // Indizes erstellen (falls noch nicht vorhanden)
        conn.execute_batch("
            CREATE INDEX IF NOT EXISTS idx_index ON my_table(\"index\" COLLATE NOCASE);
            CREATE INDEX IF NOT EXISTS idx_title ON my_table(titel COLLATE NOCASE);
            CREATE INDEX IF NOT EXISTS idx_artist ON my_table(kuenstler COLLATE NOCASE);
            CREATE INDEX IF NOT EXISTS idx_labelcode ON my_table(labelcode COLLATE NOCASE);
        ")?;
        
        self.db_connection = Some(conn);
        info!("Optimierte Verbindung zur SQLite-Datenbank hergestellt.");
        Ok(())
    }

    /// Fügt eine Datei hinzu, falls sie noch nicht vorhanden ist.
    pub fn add_file(&mut self, path: String) {
        if !self.filenames.contains(&path) {
            self.filenames.push(path.clone());
            self.selected_files.push(false);
            info!("Datei hinzugefügt: {}", path);
        } else {
            info!("Datei bereits in der Liste: {}", path);
        }
    }

    pub fn vacuum_database(&mut self) -> Result<()> {
        if let Some(conn) = &self.db_connection {
            conn.execute_batch("VACUUM;")?;
            info!("Datenbank-Vakuumierung abgeschlossen.");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Keine Datenbankverbindung vorhanden"))
        }
    }

    pub fn analyze_database(&mut self) -> Result<()> {
        if let Some(conn) = &self.db_connection {
            conn.execute_batch("ANALYZE;")?;
            info!("Datenbank-Analyse abgeschlossen.");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Keine Datenbankverbindung vorhanden"))
        }
    }

    /// Löscht alle ausgewählten Dateien.
    pub fn delete_selected_files(&mut self) {
        let expected_size = self.filenames.len() - self.selected_files.iter().filter(|&&x| x).count();
        let mut new_filenames = Vec::with_capacity(expected_size);
        let mut new_selected_files = Vec::with_capacity(expected_size);

        for (i, filename) in self.filenames.iter().enumerate() {
            if !self.selected_files[i] {
                new_filenames.push(filename.clone());
                new_selected_files.push(false);
            } else {
                info!("Datei gelöscht: {}", filename);
            }
        }

        self.filenames = new_filenames;
        self.selected_files = new_selected_files;
        let _ = crate::parser::parse_all_files(self);
    }

    /// Löscht alle Dateien.
    pub fn delete_all_files(&mut self) {
        self.filenames.clear();
        self.selected_files.clear();
        self.tracks_per_file.clear();
        info!("Alle Dateien gelöscht.");
    }

    /// Formatiert die Dauer in Sekunden in ein Format "S:MM".
    pub fn format_duration(&self, seconds: f64) -> String {
        let total_hundredths = (seconds * 100.0).round() as i64;
        let s = total_hundredths / 100;
        let ms = total_hundredths % 100;
        format!("{}:{:02}", s, ms)
    }

    /// Parst die Dateinamen/Tracks und wendet danach die Datenbank an.
    pub fn parse_filenames(&mut self) -> Result<()> {
        crate::parser::parse_all_files(self)?;
        self.apply_database_info();
        Ok(())
    }
    
    /// Sucht in der Datenbank nach passender Zeile (passend zum Index) und überschreibt Titel/Künstler/Labelcode, falls gefunden.
    fn apply_database_info(&mut self) {
        let Some(conn) = self.db_connection.as_ref() else {
            info!("Keine Datenbankverbindung vorhanden. Überspringe apply_database_info().");
            return;
        };
    
        // Prepare the query once
        let query = r#"
            SELECT kuenstler, titel, labelcode
            FROM my_table
            WHERE LOWER("index") = LOWER(?1)
            LIMIT 1
        "#;
    
        let mut stmt = match conn.prepare(query) {
            Ok(s) => {
                info!("Abfrage vorbereitet.");
                s
            },
            Err(e) => {
                info!("Fehler beim Vorbereiten der Abfrage: {}", e);
                return;
            }
        };
    
        // Iterate through all tracks and update artists from database
        for (_filename, tracks) in self.tracks_per_file.iter_mut() {
            for track in tracks.iter_mut() {
                info!("Attempting to match track: '{}'", track.titel);
                let result = stmt.query_row(params![&track.index], |row| {
                    Ok((
                        row.get::<_, String>(1)?, // Titel
                        row.get::<_, String>(0)?, // Künstler
                        row.get::<_, String>(2)?, // Labelcode
                    ))
                });
    
                match result {
                    Ok((db_title, db_kuenstler, db_labelcode)) => {
                        // **Daten mit DB-Werten überschreiben**
                        track.titel = db_title;
                        track.kuenstler = db_kuenstler;
                        track.label_code = db_labelcode;
    
                        info!(
                            "DB-Treffer: Titel='{}', Künstler='{}', Labelcode='{}'",
                            track.titel, track.kuenstler, track.label_code
                        );
                    }
                    Err(_) => {
                        info!(
                            "Kein DB-Treffer für '{}', behalte ursprüngliche Werte.",
                            track.index
                        );
                    }
                }
            }
        }
    }
}

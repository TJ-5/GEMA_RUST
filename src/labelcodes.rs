use anyhow::{Context, Result};
use log::{info, error};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn load_labelcodes(path: &str) -> Result<HashMap<String, String>> {
    if !Path::new(path).exists() {
        info!("Labelcodes-Datei '{}' nicht gefunden.", path);
        return Ok(HashMap::new());
    }

    let file = File::open(path)
        .with_context(|| format!("Kann Labelcodes-Datei '{}' nicht öffnen.", path))?;
    let reader = BufReader::new(file);

    let label_dict: HashMap<String, String> = match serde_json::from_reader(reader) {
        Ok(json_data) => {
            info!("Labelcodes erfolgreich geladen.");
            json_data
        }
        Err(e) => {
            error!("Fehler beim Parsen der Labelcodes-Datei: {}", e);
            HashMap::new()
        }
    };

    Ok(label_dict)
}

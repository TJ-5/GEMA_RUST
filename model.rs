use serde::{Deserialize, Serialize};

/// Struktur zur Speicherung der extrahierten Track-Informationen.
#[derive(Debug, Serialize, Deserialize)]
pub struct TrackInfo {
    pub index: String,
    pub titel: String,
    pub kuenstler: String,
    pub duration: Option<f64>, // Dauer in Sekunden
    pub label_code: String,    // Labelcode
}

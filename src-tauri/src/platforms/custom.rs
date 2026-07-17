//! Provider "custom" : les jeux ajoutés à la main via le bouton +.
//! Couvre deux cas avec le même mécanisme (Launch::Exec) :
//!   - un .exe hors Steam (pas d'argument)
//!   - un émulateur + une ROM (l'émulateur en `exec`, la ROM en `args`)
//!
//! Contrairement à Steam, la "plateforme" ici n'est pas fixe : chaque entrée
//! porte SON PROPRE libellé ("PC", "SNES", "PS2"...), fourni par l'utilisateur
//! à l'ajout. C'est ce qui permet au badge de rester vrai — Alan a insisté
//! pour toujours savoir d'où vient un jeu, y compris pour ceux ajoutés à la main.

use crate::library::{Art, Game, Launch};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub const ID: &str = "custom";
pub const NAME: &str = "Ajouté manuellement";

/// Préfixe qui distingue un id "custom" d'un appid Steam — c'est à ça que
/// launch_game() reconnaît qu'il doit persister lastPlayed après coup.
const ID_PREFIX: &str = "custom-";

#[derive(Serialize, Deserialize, Clone)]
struct CustomEntry {
    id: String,
    name: String,
    platform: String,
    #[serde(rename = "execPath")]
    exec_path: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(rename = "sizeBytes", default)]
    size_bytes: u64,
    #[serde(rename = "lastPlayed", default)]
    last_played: i64,
}

fn config_dir() -> Option<PathBuf> {
    // %APPDATA%/telOS — pas dans le dossier d'install : sur une install
    // Program Files, ce dossier n'est pas garanti inscriptible.
    dirs_appdata().map(|p| p.join("telOS"))
}

/// Équivalent minimal de `dirs::config_dir()` sans ajouter la dépendance :
/// on n'a besoin que d'une seule variable d'environnement Windows.
fn dirs_appdata() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

fn entries_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("custom-games.json"))
}

fn read_entries() -> Vec<CustomEntry> {
    let Some(path) = entries_path() else { return vec![] };
    let Ok(text) = fs::read_to_string(&path) else { return vec![] };
    serde_json::from_str(&text).unwrap_or_default()
}

fn write_entries(entries: &[CustomEntry]) -> Result<(), String> {
    let dir = config_dir().ok_or("Dossier de config introuvable (%APPDATA% absent).")?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = entries_path().unwrap();
    let text = serde_json::to_string_pretty(entries).map_err(|e| e.to_string())?;
    fs::write(&path, text).map_err(|e| e.to_string())
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Toujours présent : ce n'est pas un logiciel tiers à détecter, juste un
/// fichier local qui peut être vide.
pub fn detect() -> bool {
    true
}

pub fn scan() -> Result<Vec<Game>, String> {
    Ok(read_entries()
        .into_iter()
        .map(|e| Game {
            platform: e.platform,
            id: e.id,
            name: e.name,
            launch: Launch::Exec { path: e.exec_path, args: e.args },
            size_bytes: e.size_bytes,
            last_played: e.last_played,
            art: Art::default(), // rien en local : CDN Steam ne s'applique pas hors Steam,
                                  // SteamGridDB (étape 5) est la vraie source pour ceux-ci.
        })
        .collect())
}

/// Ajoute un jeu. Revalide le chemin sur disque même s'il vient du sélecteur
/// natif : défense en profondeur, un renderer compromis ne doit pas pouvoir
/// faire persister une entrée vers un exécutable arbitraire non vérifié.
pub fn add(name: String, platform: String, exec_path: String, args: Vec<String>) -> Result<Game, String> {
    if !PathBuf::from(&exec_path).is_file() {
        return Err("Le fichier sélectionné n'existe pas ou n'est pas accessible.".to_string());
    }

    let mut entries = read_entries();

    // Doublon exact (même exe, mêmes arguments) : on ne l'ajoute pas deux fois.
    if entries.iter().any(|e| e.exec_path == exec_path && e.args == args) {
        return Err("Ce jeu est déjà dans la bibliothèque.".to_string());
    }

    let size_bytes = fs::metadata(&exec_path).map(|m| m.len()).unwrap_or(0);
    let added_at = now();

    // Id stable dérivé du chemin — pas besoin d'une dépendance UUID pour ça.
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    exec_path.hash(&mut hasher);
    args.hash(&mut hasher);
    let id = format!("{ID_PREFIX}{:x}", hasher.finish());

    let entry = CustomEntry {
        id: id.clone(),
        name: name.clone(),
        platform: platform.clone(),
        exec_path: exec_path.clone(),
        args: args.clone(),
        size_bytes,
        // Ajouté maintenant -> apparaît tout de suite dans le rail par récence
        // (loi n°4) au lieu de rester coincé tout en bas avec "jamais lancé".
        last_played: added_at,
    };
    entries.push(entry);
    write_entries(&entries)?;

    Ok(Game {
        platform,
        id,
        name,
        launch: Launch::Exec { path: exec_path, args },
        size_bytes,
        last_played: added_at,
        art: Art::default(),
    })
}

/// Un id "custom-..." vient forcément de ce provider — c'est le seul endroit
/// qui les génère (voir ID_PREFIX). launch_game() s'en sert pour savoir sur
/// quelles entrées persister lastPlayed après un lancement réussi.
pub fn is_custom_id(id: &str) -> bool {
    id.starts_with(ID_PREFIX)
}

pub fn mark_played(id: &str) {
    let mut entries = read_entries();
    if let Some(e) = entries.iter_mut().find(|e| e.id == id) {
        e.last_played = now();
        let _ = write_entries(&entries); // best-effort : un raté ici ne doit jamais casser le lancement
    }
}

pub fn remove(id: &str) -> Result<(), String> {
    let mut entries = read_entries();
    let before = entries.len();
    entries.retain(|e| e.id != id);
    if entries.len() == before {
        return Err("Jeu introuvable.".to_string());
    }
    write_entries(&entries)
}

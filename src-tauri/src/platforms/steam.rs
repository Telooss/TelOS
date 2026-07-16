//! Provider Steam. Port Rust de scripts/platforms/steam.js — même logique,
//! même filtre anti-faux-jeux, même recherche de jaquettes dans le cache local.

use crate::library::{Art, Game};
use crate::vdf;
use std::fs;
use std::path::{Path, PathBuf};

pub const ID: &str = "steam";
pub const NAME: &str = "Steam";

/// Ce que Steam installe mais qui n'est pas un jeu : runtimes, redistribuables.
/// Ils ont un appmanifest exactement comme un jeu — seul un filtre explicite les distingue.
const NON_GAME_APPIDS: &[&str] = &[
    "228980",  // Steamworks Common Redistributables
    "1070560", // Steam Linux Runtime
    "1391110", // Steam Linux Runtime - Soldier
    "1628350", // Steam Linux Runtime 3.0 (Sniper)
    "1493710", // Proton Experimental
    "2180100", // Proton Hotfix
];

fn is_non_game_name(name: &str) -> bool {
    let n = name.to_lowercase();
    n.contains("redistributable")
        || n.contains("steam linux runtime")
        || n.starts_with("proton")
        || n.starts_with("steamworks")
}

fn find_steam_root() -> Option<PathBuf> {
    let pf86 = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into());
    let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
    let candidates = [
        PathBuf::from(format!("{pf86}\\Steam")),
        PathBuf::from(format!("{pf}\\Steam")),
        PathBuf::from(r"C:\Steam"),
    ];
    candidates
        .into_iter()
        .find(|c| c.join("steamapps").join("libraryfolders.vdf").exists())
}

pub fn detect() -> bool {
    find_steam_root().is_some()
}

/// Steam peut avoir des bibliothèques sur plusieurs disques.
fn find_library_folders(steam_root: &Path) -> Vec<PathBuf> {
    let vdf_path = steam_root.join("steamapps").join("libraryfolders.vdf");
    let text = match fs::read_to_string(&vdf_path) {
        Ok(t) => t,
        Err(_) => return vec![steam_root.to_path_buf()],
    };
    let parsed = vdf::parse(&text);

    let mut paths = Vec::new();
    if let Some(folders) = parsed.get("libraryfolders").and_then(|v| v.as_map()) {
        for (key, val) in folders {
            if !key.chars().all(|c| c.is_ascii_digit()) {
                continue; // les entrées sont indexées "0", "1", "2"...
            }
            let p = match val {
                vdf::Value::Str(s) => Some(s.clone()),
                vdf::Value::Map(m) => m.get("path").and_then(|v| v.as_str()).map(str::to_string),
            };
            if let Some(p) = p {
                paths.push(PathBuf::from(p));
            }
        }
    }
    if !paths.iter().any(|p| p == steam_root) {
        paths.insert(0, steam_root.to_path_buf()); // le dossier racine n'est pas toujours listé
    }
    paths
}

/// Collecte récursivement les fichiers sous `dir`, jusqu'à `depth` niveaux.
fn collect_files(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth == 0 {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.filter_map(|e| e.ok()) {
            let p = e.path();
            if p.is_dir() {
                collect_files(&p, depth - 1, out);
            } else {
                out.push(p);
            }
        }
    }
}

/// `library_hero_blur.jpg` contient `library_hero` : sans exclusion explicite,
/// on servirait la version floue à la place du vrai visuel.
fn is_kind(fname: &str, kind: &str) -> bool {
    match kind {
        "portrait" => fname.contains("library_600x900"),
        "hero" => fname.contains("library_hero") && !fname.contains("library_hero_blur"),
        "logo" => fname.contains("logo"),
        _ => false,
    }
}

/// Steam a empilé trois structures de cache au fil des versions :
///   librarycache/<appid>_library_hero.jpg          (à plat, ancien)
///   librarycache/<appid>/library_hero.jpg          (2 niveaux)
///   librarycache/<appid>/<hash>/library_hero.jpg   (3 niveaux, récent)
/// Les trois coexistent sur une même machine — d'où la recherche récursive.
fn find_artwork(steam_root: &Path, appid: &str) -> Art {
    let cache = steam_root.join("appcache").join("librarycache");
    if !cache.exists() {
        return Art::default();
    }

    let mut candidates: Vec<PathBuf> = Vec::new();

    // Structures 2 et 3 niveaux : tout ce qui vit sous librarycache/<appid>/
    let app_dir = cache.join(appid);
    if app_dir.is_dir() {
        collect_files(&app_dir, 3, &mut candidates);
    }

    // Structure à plat : librarycache/<appid>_library_hero.jpg
    if let Ok(entries) = fs::read_dir(&cache) {
        let prefix = format!("{appid}_");
        for e in entries.filter_map(|e| e.ok()) {
            if e.file_name().to_string_lossy().starts_with(&prefix) {
                let p = e.path();
                if p.is_file() {
                    candidates.push(p);
                }
            }
        }
    }

    let pick = |kind: &str| -> Option<String> {
        candidates
            .iter()
            .find(|p| {
                let f = p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                (f.ends_with(".jpg") || f.ends_with(".png")) && is_kind(&f, kind)
            })
            .map(|p| p.to_string_lossy().to_string())
    };

    Art {
        portrait: pick("portrait"),
        hero: pick("hero"),
        logo: pick("logo"),
    }
}

fn read_games(steam_root: &Path, library_path: &Path) -> Vec<Game> {
    let apps_dir = library_path.join("steamapps");
    let entries = match fs::read_dir(&apps_dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    entries
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let fname = entry.file_name().to_string_lossy().to_string();
            if !(fname.starts_with("appmanifest_") && fname.ends_with(".acf")) {
                return None;
            }
            // Un manifeste corrompu ne doit pas faire tomber tout le scan.
            let text = fs::read_to_string(entry.path()).ok()?;
            let parsed = vdf::parse(&text);
            let state = parsed.get("AppState")?.as_map()?;
            let appid = state.get("appid")?.as_str()?.to_string();
            let name = state.get("name")?.as_str()?.to_string();
            let size_bytes: u64 = state
                .get("SizeOnDisk")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let last_played: i64 = state
                .get("LastPlayed")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            Some(Game {
                platform: ID.to_string(),
                id: appid.clone(),
                name,
                launch: format!("steam://rungameid/{appid}"),
                size_bytes,
                last_played,
                art: find_artwork(steam_root, &appid),
            })
        })
        .filter(|g| !NON_GAME_APPIDS.contains(&g.id.as_str()) && !is_non_game_name(&g.name))
        .collect()
}

pub fn scan() -> Result<Vec<Game>, String> {
    let steam_root = find_steam_root().ok_or_else(|| "Steam introuvable sur cette machine.".to_string())?;
    let libraries = find_library_folders(&steam_root);
    let mut games: Vec<Game> = libraries.iter().flat_map(|lib| read_games(&steam_root, lib)).collect();
    games.sort_by(|a, b| b.last_played.cmp(&a.last_played));
    Ok(games)
}

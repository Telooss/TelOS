//! Téléchargement de jaquettes en arrière-plan.
//!
//! Deux sources, dans cet ordre de priorité :
//!   1. CDN Steam public (gratuit, sans clé, sans compte) — jeux Steam uniquement.
//!   2. SteamGridDB (recherche floue par nom) — filet universel, clé requise.
//!
//! Principes (docs/plan-bibliotheque.md) :
//!   - Le réseau n'est JAMAIS sur le chemin critique : le boot affiche ce qui est local.
//!   - Une jaquette téléchargée l'est une fois pour toutes (cache disque permanent).
//!   - Cache négatif obligatoire (.404) pour ne pas réinterroger en boucle.
//!   - L'IPC transporte des CHEMINS, jamais des octets.

use crate::library::{Art, Game};
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;

/// Empêche de lancer plusieurs threads de téléchargement en parallèle.
static DOWNLOADING: AtomicBool = AtomicBool::new(false);

// ---------------------------------------------------------------------------
// Cache disque
// ---------------------------------------------------------------------------

/// Racine du cache : %APPDATA%/telOS/cache/
fn cache_root() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("telOS").join("cache"))
}

/// Dossier de cache pour un jeu donné : cache/<platform>/<id>/
fn cache_dir(platform: &str, id: &str) -> Option<PathBuf> {
    cache_root().map(|r| r.join(platform).join(id))
}

/// Vérifie si un visuel est déjà dans le cache local (toute extension).
fn cached_path(platform: &str, id: &str, kind: &str) -> Option<PathBuf> {
    let dir = cache_dir(platform, id)?;
    // SteamGridDB peut renvoyer du PNG, WebP ou JPG — on cherche tout.
    for ext in &["jpg", "png", "webp"] {
        let file = dir.join(format!("{kind}.{ext}"));
        if file.exists() {
            return Some(file);
        }
    }
    None
}

/// Vérifie si un marqueur négatif existe (on a déjà cherché et rien trouvé).
fn has_negative_cache(platform: &str, id: &str, kind: &str) -> bool {
    cache_dir(platform, id)
        .map(|d| d.join(format!("{kind}.404")).exists())
        .unwrap_or(false)
}

/// Écrit le marqueur négatif.
fn write_negative(platform: &str, id: &str, kind: &str) {
    if let Some(dir) = cache_dir(platform, id) {
        let _ = fs::create_dir_all(&dir);
        let _ = fs::write(dir.join(format!("{kind}.404")), b"");
    }
}

/// Sauvegarde une image téléchargée et renvoie le chemin.
/// `ext` est l'extension réelle détectée depuis l'URL source (jpg, png, webp).
fn save_image(platform: &str, id: &str, kind: &str, ext: &str, data: &[u8]) -> Option<PathBuf> {
    let dir = cache_dir(platform, id)?;
    fs::create_dir_all(&dir).ok()?;
    let path = dir.join(format!("{kind}.{ext}"));
    fs::write(&path, data).ok()?;
    Some(path)
}

/// Complète les visuels d'un jeu avec le cache local (appelé au scan).
pub fn fill_from_cache(game: &mut Game) {
    if game.art.portrait.is_none() {
        game.art.portrait = cached_path(&game.platform, &game.id, "portrait")
            .map(|p| p.to_string_lossy().to_string());
    }
    if game.art.hero.is_none() {
        game.art.hero = cached_path(&game.platform, &game.id, "hero")
            .map(|p| p.to_string_lossy().to_string());
    }
    if game.art.logo.is_none() {
        game.art.logo = cached_path(&game.platform, &game.id, "logo")
            .map(|p| p.to_string_lossy().to_string());
    }
}

// ---------------------------------------------------------------------------
// Téléchargement HTTP
// ---------------------------------------------------------------------------

/// Résultat d'un téléchargement : octets bruts + extension détectée.
struct Downloaded {
    bytes: Vec<u8>,
    ext: String, // "jpg", "png", "webp"
}

/// Déduit l'extension depuis l'URL (ou le Content-Type en fallback).
fn guess_ext(url: &str, content_type: Option<&str>) -> String {
    // 1. Extension depuis l'URL (avant tout query string)
    let path = url.split('?').next().unwrap_or(url);
    if path.ends_with(".png") { return "png".into(); }
    if path.ends_with(".webp") { return "webp".into(); }
    if path.ends_with(".jpg") || path.ends_with(".jpeg") { return "jpg".into(); }
    // 2. Fallback : Content-Type
    if let Some(ct) = content_type {
        if ct.contains("png") { return "png".into(); }
        if ct.contains("webp") { return "webp".into(); }
    }
    "jpg".into()
}

/// Télécharge une image par URL. Renvoie les octets bruts + extension détectée.
fn download(url: &str, api_key: Option<&str>) -> Option<Downloaded> {
    let mut req = ureq::get(url);
    if let Some(key) = api_key {
        req = req.set("Authorization", &format!("Bearer {key}"));
    }
    let resp = req.call().ok()?;
    if resp.status() != 200 {
        return None;
    }
    let ct = resp.header("content-type").map(String::from);
    let ext = guess_ext(url, ct.as_deref());
    // Limite à 10 Mo pour éviter de tout avaler en cas de réponse inattendue.
    let mut buf = Vec::new();
    resp.into_reader().take(10_000_000).read_to_end(&mut buf).ok()?;
    // Rejette les réponses trop petites (< 1 Ko) : probablement une erreur HTML.
    if buf.len() < 1024 {
        return None;
    }
    Some(Downloaded { bytes: buf, ext })
}

// ---------------------------------------------------------------------------
// CDN Steam (source n°1 — gratuit, sans clé)
// ---------------------------------------------------------------------------

/// Portrait depuis le CDN Steam public.
fn cdn_steam_portrait(appid: &str) -> Option<Downloaded> {
    let url = format!(
        "https://cdn.cloudflare.steamstatic.com/steam/apps/{appid}/library_600x900.jpg"
    );
    download(&url, None)
}

/// Hero depuis le CDN Steam public.
fn cdn_steam_hero(appid: &str) -> Option<Downloaded> {
    let url = format!(
        "https://cdn.cloudflare.steamstatic.com/steam/apps/{appid}/library_hero.jpg"
    );
    download(&url, None)
}

/// Logo depuis le CDN Steam public.
fn cdn_steam_logo(appid: &str) -> Option<Downloaded> {
    let url = format!(
        "https://cdn.cloudflare.steamstatic.com/steam/apps/{appid}/logo.png"
    );
    download(&url, None)
}

// ---------------------------------------------------------------------------
// SteamGridDB (source n°2 — clé requise)
// ---------------------------------------------------------------------------

/// Cherche l'URL de la première image renvoyée par un endpoint SteamGridDB.
fn sgdb_first_image_url(endpoint: &str, api_key: &str) -> Option<String> {
    let url = format!("https://www.steamgriddb.com/api/v2{endpoint}");
    let mut req = ureq::get(&url);
    req = req.set("Authorization", &format!("Bearer {api_key}"));
    let resp = req.call().ok()?;
    if resp.status() != 200 {
        return None;
    }
    let body: serde_json::Value = resp.into_json().ok()?;
    body.get("data")?
        .as_array()?
        .first()?
        .get("url")?
        .as_str()
        .map(String::from)
}

/// Cherche l'ID SteamGridDB d'un jeu par son nom (recherche floue).
fn sgdb_search_game(name: &str, api_key: &str) -> Option<u64> {
    let url = format!(
        "https://www.steamgriddb.com/api/v2/search/autocomplete/{}",
        urlencoded(name)
    );
    let resp = ureq::get(&url)
        .set("Authorization", &format!("Bearer {api_key}"))
        .call()
        .ok()?;
    if resp.status() != 200 {
        return None;
    }
    let body: serde_json::Value = resp.into_json().ok()?;
    body.get("data")?
        .as_array()?
        .first()?
        .get("id")?
        .as_u64()
}

/// Télécharge un visuel depuis SteamGridDB pour un jeu Steam (par appid).
fn sgdb_by_steam(appid: &str, kind: &str, api_key: &str) -> Option<Downloaded> {
    let endpoint = match kind {
        "portrait" => format!("/grids/steam/{appid}?dimensions=600x900"),
        "hero" => format!("/heroes/steam/{appid}"),
        "logo" => format!("/logos/steam/{appid}"),
        _ => return None,
    };
    let img_url = sgdb_first_image_url(&endpoint, api_key)?;
    download(&img_url, None)
}

/// Télécharge un visuel depuis SteamGridDB pour un jeu custom (par game ID interne SGDB).
fn sgdb_by_game_id(game_id: u64, kind: &str, api_key: &str) -> Option<Downloaded> {
    let endpoint = match kind {
        "portrait" => format!("/grids/game/{game_id}?dimensions=600x900"),
        "hero" => format!("/heroes/game/{game_id}"),
        "logo" => format!("/logos/game/{game_id}"),
        _ => return None,
    };
    let img_url = sgdb_first_image_url(&endpoint, api_key)?;
    download(&img_url, None)
}

/// Encodage URL minimal (espaces et caractères non-ASCII).
fn urlencoded(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Chargement de la clé API
// ---------------------------------------------------------------------------

fn load_api_key() -> Option<String> {
    // D'abord le fichier local de dev (src-tauri/config.local.json)
    let dev_path = std::env::current_dir()
        .ok()?
        .join("config.local.json");
    if let Some(key) = read_key_from(&dev_path) {
        return Some(key);
    }
    // Puis le fichier en prod (%APPDATA%/telOS/config.local.json)
    let prod_path = std::env::var_os("APPDATA")
        .map(|a| PathBuf::from(a).join("telOS").join("config.local.json"))?;
    read_key_from(&prod_path)
}

fn read_key_from(path: &PathBuf) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("steamgriddb_api_key")?
        .as_str()
        .filter(|s| !s.is_empty() && !s.starts_with("obtenir"))
        .map(String::from)
}

// ---------------------------------------------------------------------------
// Payload de l'événement Tauri
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
pub struct ArtUpdate {
    pub platform: String,
    pub id: String,
    pub art: Art,
}

// ---------------------------------------------------------------------------
// Point d'entrée : lance le downloader en arrière-plan
// ---------------------------------------------------------------------------

/// Lance le téléchargement des jaquettes manquantes dans un thread dédié.
/// Ne fait rien si un téléchargement est déjà en cours.
/// Émet un événement `game-art-ready` pour chaque jeu mis à jour.
pub fn spawn_if_needed(app: tauri::AppHandle, games: Vec<Game>) {
    // Un seul thread à la fois.
    if DOWNLOADING.swap(true, Ordering::SeqCst) {
        return;
    }

    std::thread::spawn(move || {
        let api_key = load_api_key();
        if api_key.is_none() {
            eprintln!("[telos/art] Pas de clé SteamGridDB — seul le CDN Steam sera utilisé.");
        }

        for game in &games {
            let needs_portrait = game.art.portrait.is_none()
                && !has_negative_cache(&game.platform, &game.id, "portrait");
            let needs_hero = game.art.hero.is_none()
                && !has_negative_cache(&game.platform, &game.id, "hero");
            let needs_logo = game.art.logo.is_none()
                && !has_negative_cache(&game.platform, &game.id, "logo");

            if !needs_portrait && !needs_hero && !needs_logo {
                continue;
            }

            eprintln!(
                "[telos/art] {} ({}/{}): portrait={} hero={} logo={}",
                game.name,
                game.platform,
                game.id,
                if needs_portrait { "manquant" } else { "ok" },
                if needs_hero { "manquant" } else { "ok" },
                if needs_logo { "manquant" } else { "ok" },
            );

            let mut updated_art = Art {
                portrait: game.art.portrait.clone(),
                hero: game.art.hero.clone(),
                logo: game.art.logo.clone(),
            };
            let mut changed = false;

            // --- Source 1 : CDN Steam (jeux Steam uniquement) ---
            if game.platform == "steam" {
                for (kind, needs, setter) in [
                    ("portrait", needs_portrait, &mut updated_art.portrait as &mut Option<String>),
                    ("hero", needs_hero, &mut updated_art.hero),
                    ("logo", needs_logo, &mut updated_art.logo),
                ] {
                    if !needs {
                        continue;
                    }
                    let data = match kind {
                        "portrait" => cdn_steam_portrait(&game.id),
                        "hero" => cdn_steam_hero(&game.id),
                        "logo" => cdn_steam_logo(&game.id),
                        _ => None,
                    };
                    if let Some(dl) = data {
                        if let Some(path) = save_image(&game.platform, &game.id, kind, &dl.ext, &dl.bytes) {
                            eprintln!("[telos/art]   ✓ CDN Steam {kind} (.{}) → {}", dl.ext, path.display());
                            *setter = Some(path.to_string_lossy().to_string());
                            changed = true;
                        }
                    }
                    // On ne marque PAS en négatif ici : SteamGridDB pourrait l'avoir.
                }
            }

            // --- Source 2 : SteamGridDB (filet universel) ---
            if let Some(ref key) = api_key {
                // Pour les jeux Steam, on peut requêter directement par appid.
                // Pour les jeux custom, on cherche par nom d'abord.
                let sgdb_game_id: Option<u64> = if game.platform == "steam" {
                    None // on utilise l'endpoint /steam/ directement
                } else {
                    sgdb_search_game(&game.name, key)
                };

                for (kind, field) in [
                    ("portrait", &mut updated_art.portrait as &mut Option<String>),
                    ("hero", &mut updated_art.hero),
                    ("logo", &mut updated_art.logo),
                ] {
                    // Ne cherche que si toujours manquant après le CDN.
                    if field.is_some() || has_negative_cache(&game.platform, &game.id, kind) {
                        continue;
                    }

                    let data = if game.platform == "steam" {
                        sgdb_by_steam(&game.id, kind, key)
                    } else if let Some(gid) = sgdb_game_id {
                        sgdb_by_game_id(gid, kind, key)
                    } else {
                        None
                    };

                    match data {
                        Some(dl) => {
                            if let Some(path) =
                                save_image(&game.platform, &game.id, kind, &dl.ext, &dl.bytes)
                            {
                                eprintln!(
                                    "[telos/art]   ✓ SteamGridDB {kind} (.{}) → {}",
                                    dl.ext,
                                    path.display()
                                );
                                *field = Some(path.to_string_lossy().to_string());
                                changed = true;
                            }
                        }
                        None => {
                            // Aucune source n'a rien : cache négatif.
                            write_negative(&game.platform, &game.id, kind);
                            eprintln!("[telos/art]   ✗ {kind} introuvable, marqué .404");
                        }
                    }
                }
            } else {
                // Pas de clé SGDB : cache négatif pour les visuels non couverts par le CDN.
                for (kind, field) in [
                    ("portrait", &updated_art.portrait),
                    ("hero", &updated_art.hero),
                    ("logo", &updated_art.logo),
                ] {
                    if field.is_none() && !has_negative_cache(&game.platform, &game.id, kind) {
                        // Pour Steam, le CDN a déjà été tenté — on marque en négatif.
                        if game.platform == "steam" {
                            write_negative(&game.platform, &game.id, kind);
                        }
                        // Pour les custom sans clé SGDB, on ne peut rien faire.
                    }
                }
            }

            if changed {
                let _ = app.emit("game-art-ready", ArtUpdate {
                    platform: game.platform.clone(),
                    id: game.id.clone(),
                    art: updated_art,
                });
            }
        }

        DOWNLOADING.store(false, Ordering::SeqCst);
        eprintln!("[telos/art] Téléchargement terminé.");
    });
}

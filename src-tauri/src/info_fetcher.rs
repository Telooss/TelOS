//! Fiche jeu : résumé, développeur, genres... Même discipline que
//! art_downloader.rs, plus une couche que celui-ci n'a pas — la surcharge
//! utilisateur (Alan peut corriger ce qui a été auto-récupéré, y compris
//! remplacer la jaquette elle-même).
//!
//! Deux sources, dans cet ordre de priorité :
//!   1. L'API officielle Steam (`store.steampowered.com/api/appdetails`) —
//!      gratuite, sans clé, jeux Steam uniquement.
//!   2. RAWG (recherche floue par nom) — filet universel, clé requise.
//!
//! Contrairement aux jaquettes, le réseau n'est PAS forcément hors du
//! chemin critique ici : ouvrir la fiche est une action délibérée de
//! l'utilisateur, qui attend du contenu tout de suite — comme une page de
//! store. On accepte donc un aller-retour réseau synchrone à l'ouverture,
//! mais jamais au boot, et jamais deux fois pour le même jeu (cache permanent).

use crate::art_downloader::{load_api_key, urlencoded};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct GameInfo {
    pub summary: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(rename = "releaseDate")]
    pub release_date: Option<String>,
    // Note critique — Metacritic côté Steam ET RAWG, même échelle (/100),
    // donc un seul champ suffit pour les deux sources.
    pub metacritic: Option<u32>,
    // Temps de jeu moyen constaté par la communauté RAWG. Absent des
    // métadonnées de store Steam (ce n'est pas une donnée "store", donc
    // toujours None pour game.platform == "steam" — normal, pas un bug).
    #[serde(rename = "playtimeHours")]
    pub playtime_hours: Option<u32>,
    // Chemin local choisi par l'utilisateur pour remplacer la jaquette
    // auto-détectée. None = on garde l'art normal (Steam/cache/SteamGridDB).
    #[serde(rename = "portraitOverride", skip_serializing_if = "Option::is_none")]
    pub portrait_override: Option<String>,
}

// ---------------------------------------------------------------------------
// Cache (auto-récupéré, jamais modifié par l'utilisateur directement)
// ---------------------------------------------------------------------------

fn info_cache_path(platform: &str, id: &str) -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|a| {
        PathBuf::from(a)
            .join("telOS")
            .join("cache")
            .join(platform)
            .join(id)
            .join("info.json")
    })
}

fn read_cached_info(platform: &str, id: &str) -> Option<GameInfo> {
    let path = info_cache_path(platform, id)?;
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_cached_info(platform: &str, id: &str, info: &GameInfo) {
    let Some(path) = info_cache_path(platform, id) else { return };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(text) = serde_json::to_string_pretty(info) {
        let _ = fs::write(path, text);
    }
}

// ---------------------------------------------------------------------------
// Surcharges utilisateur (overrides.json) : gagnent TOUJOURS, champ par champ
// ---------------------------------------------------------------------------

fn overrides_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("telOS").join("overrides.json"))
}

fn read_overrides() -> HashMap<String, GameInfo> {
    let Some(path) = overrides_path() else { return HashMap::new() };
    let Ok(text) = fs::read_to_string(path) else { return HashMap::new() };
    serde_json::from_str(&text).unwrap_or_default()
}

fn write_overrides(map: &HashMap<String, GameInfo>) -> Result<(), String> {
    let path = overrides_path().ok_or("Dossier de config introuvable (%APPDATA% absent).")?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(map).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

fn override_key(platform: &str, id: &str) -> String {
    format!("{platform}/{id}")
}

/// Fusion champ par champ : la surcharge gagne partout où elle est renseignée.
/// Jamais tout-ou-rien — corriger juste le développeur ne doit pas effacer
/// un résumé auto-récupéré qui était déjà bon.
fn merge(base: GameInfo, over: Option<&GameInfo>) -> GameInfo {
    let Some(o) = over else { return base };
    GameInfo {
        summary: o.summary.clone().or(base.summary),
        developer: o.developer.clone().or(base.developer),
        publisher: o.publisher.clone().or(base.publisher),
        genres: if o.genres.is_empty() { base.genres } else { o.genres.clone() },
        release_date: o.release_date.clone().or(base.release_date),
        metacritic: o.metacritic.or(base.metacritic),
        playtime_hours: o.playtime_hours.or(base.playtime_hours),
        portrait_override: o.portrait_override.clone(),
    }
}

// ---------------------------------------------------------------------------
// Source 1 : API officielle Steam (gratuite, sans clé)
// ---------------------------------------------------------------------------

fn fetch_steam(appid: &str) -> Option<GameInfo> {
    let url = format!("https://store.steampowered.com/api/appdetails?appids={appid}&l=french");
    let resp = ureq::get(&url).call().ok()?;
    if resp.status() != 200 {
        return None;
    }
    let body: serde_json::Value = resp.into_json().ok()?;
    let entry = body.get(appid)?;
    if !entry.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
        return None;
    }
    let data = entry.get("data")?;

    let genres = data
        .get("genres")
        .and_then(|g| g.as_array())
        .map(|arr| arr.iter().filter_map(|g| g.get("description")?.as_str().map(String::from)).collect())
        .unwrap_or_default();

    Some(GameInfo {
        summary: data.get("short_description").and_then(|v| v.as_str()).map(String::from),
        developer: data
            .get("developers")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .map(String::from),
        publisher: data
            .get("publishers")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .map(String::from),
        genres,
        release_date: data
            .get("release_date")
            .and_then(|v| v.get("date"))
            .and_then(|v| v.as_str())
            .map(String::from),
        // Steam imbrique le score sous metacritic.score — RAWG le renvoie à
        // plat, d'où la différence de traitement entre les deux sources.
        metacritic: data.get("metacritic").and_then(|v| v.get("score")).and_then(|v| v.as_u64()).map(|v| v as u32),
        playtime_hours: None, // pas une donnée de store Steam, seulement côté RAWG
        portrait_override: None,
    })
}

// ---------------------------------------------------------------------------
// Source 2 : RAWG (recherche floue par nom, clé requise)
// ---------------------------------------------------------------------------

fn fetch_rawg(name: &str, api_key: &str) -> Option<GameInfo> {
    // Étape 1 : recherche par nom -> id RAWG (même schéma que la recherche
    // SteamGridDB pour les jaquettes hors Steam).
    let search_url = format!(
        "https://api.rawg.io/api/games?search={}&key={api_key}&page_size=1",
        urlencoded(name)
    );
    let resp = ureq::get(&search_url).call().ok()?;
    if resp.status() != 200 {
        return None;
    }
    let body: serde_json::Value = resp.into_json().ok()?;
    let rawg_id = body.get("results")?.as_array()?.first()?.get("id")?.as_u64()?;

    // Étape 2 : fiche détaillée -> développeur/éditeur/résumé (absents de la recherche).
    let detail_url = format!("https://api.rawg.io/api/games/{rawg_id}?key={api_key}");
    let resp2 = ureq::get(&detail_url).call().ok()?;
    if resp2.status() != 200 {
        return None;
    }
    let detail: serde_json::Value = resp2.into_json().ok()?;

    let genres = detail
        .get("genres")
        .and_then(|g| g.as_array())
        .map(|arr| arr.iter().filter_map(|g| g.get("name")?.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let name_of = |field: &str| -> Option<String> {
        detail
            .get(field)
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .map(String::from)
    };
    // description_raw peut faire plusieurs paragraphes : le premier suffit
    // pour une fiche, pas besoin d'un roman.
    let summary = detail
        .get("description_raw")
        .and_then(|v| v.as_str())
        .map(|s| s.split("\n\n").next().unwrap_or(s).trim().to_string())
        .filter(|s| !s.is_empty());

    Some(GameInfo {
        summary,
        developer: name_of("developers"),
        publisher: name_of("publishers"),
        genres,
        release_date: detail.get("released").and_then(|v| v.as_str()).map(String::from),
        metacritic: detail.get("metacritic").and_then(|v| v.as_u64()).map(|v| v as u32),
        playtime_hours: detail.get("playtime").and_then(|v| v.as_u64()).map(|v| v as u32).filter(|&h| h > 0),
        portrait_override: None,
    })
}

// ---------------------------------------------------------------------------
// Points d'entrée publics
// ---------------------------------------------------------------------------

/// Lit le cache si présent, sinon interroge la bonne source et écrit le
/// résultat une fois pour toutes. Fusionné avec la surcharge utilisateur
/// avant d'être renvoyé — jamais l'inverse, pour que save_override() reste
/// la seule source de vérité sur ce qu'Alan a corrigé à la main.
pub fn get_or_fetch(platform: &str, id: &str, name: &str) -> GameInfo {
    let base = match read_cached_info(platform, id) {
        Some(cached) => cached,
        None => {
            let fetched = if platform == "steam" {
                fetch_steam(id)
            } else {
                load_api_key("rawg_api_key").and_then(|key| fetch_rawg(name, &key))
            }
            .unwrap_or_default();
            write_cached_info(platform, id, &fetched);
            fetched
        }
    };
    let overrides = read_overrides();
    merge(base, overrides.get(&override_key(platform, id)))
}

/// Enregistre une correction manuelle. Fusion avec ce qui existe déjà pour
/// cette entrée — modifier un seul champ ne touche pas aux autres.
///
/// ⚠️ Limite connue : un champ à `None` dans `patch` ne peut pas EFFACER une
/// surcharge déjà posée (il retombe sur l'ancienne valeur). Suffisant pour
/// corriger/compléter ; revenir en arrière demandera un bouton dédié plus tard.
pub fn save_override(platform: &str, id: &str, patch: GameInfo) -> Result<(), String> {
    let mut all = read_overrides();
    let key = override_key(platform, id);
    let existing = all.remove(&key).unwrap_or_default();
    let merged = GameInfo {
        summary: patch.summary.or(existing.summary),
        developer: patch.developer.or(existing.developer),
        publisher: patch.publisher.or(existing.publisher),
        genres: if patch.genres.is_empty() { existing.genres } else { patch.genres },
        release_date: patch.release_date.or(existing.release_date),
        metacritic: patch.metacritic.or(existing.metacritic),
        playtime_hours: patch.playtime_hours.or(existing.playtime_hours),
        portrait_override: patch.portrait_override.or(existing.portrait_override),
    };
    all.insert(key, merged);
    write_overrides(&all)
}

/// Applique une surcharge de jaquette éventuelle sur un jeu déjà scanné.
/// Appelé depuis library.rs juste après fill_from_cache() — la surcharge
/// utilisateur gagne toujours sur l'art auto-détecté.
pub fn apply_portrait_override(game: &mut crate::library::Game) {
    let overrides = read_overrides();
    if let Some(o) = overrides.get(&override_key(&game.platform, &game.id)) {
        if let Some(p) = &o.portrait_override {
            game.art.portrait = Some(p.clone());
        }
    }
}

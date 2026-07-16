//! La bibliothèque unifiée : agrège tous les providers de plateforme.
//! Miroir Rust de scripts/library.js. Aujourd'hui seul Steam est câblé,
//! mais chaque jeu porte déjà sa plateforme d'origine — ajouter un provider
//! revient à déposer un module dans platforms/ et à l'ajouter à scan_all().

use crate::platforms::steam;
use serde::Serialize;

#[derive(Serialize, Clone, Default)]
pub struct Art {
    pub portrait: Option<String>,
    pub hero: Option<String>,
    pub logo: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct Game {
    pub platform: String,
    pub id: String,
    pub name: String,
    pub launch: String,
    #[serde(rename = "sizeBytes")]
    pub size_bytes: u64,
    #[serde(rename = "lastPlayed")]
    pub last_played: i64,
    pub art: Art,
}

#[derive(Serialize)]
pub struct PlatformStatus {
    pub id: String,
    pub name: String,
    pub present: bool,
    pub count: usize,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct ScanResult {
    pub games: Vec<Game>,
    pub platforms: Vec<PlatformStatus>,
}

/// Un provider en échec est signalé, jamais fatal aux autres :
/// une bibliothèque partielle vaut mieux qu'un écran vide.
pub fn scan_all() -> ScanResult {
    let mut games = Vec::new();
    let mut platforms = Vec::new();

    if steam::detect() {
        match steam::scan() {
            Ok(g) => {
                platforms.push(PlatformStatus {
                    id: steam::ID.into(),
                    name: steam::NAME.into(),
                    present: true,
                    count: g.len(),
                    error: None,
                });
                games.extend(g);
            }
            Err(e) => platforms.push(PlatformStatus {
                id: steam::ID.into(),
                name: steam::NAME.into(),
                present: true,
                count: 0,
                error: Some(e),
            }),
        }
    } else {
        platforms.push(PlatformStatus {
            id: steam::ID.into(),
            name: steam::NAME.into(),
            present: false,
            count: 0,
            error: None,
        });
    }

    // Tri par récence, toutes plateformes confondues — loi n°4.
    games.sort_by(|a, b| b.last_played.cmp(&a.last_played));
    ScanResult { games, platforms }
}

pub fn find_game<'a>(games: &'a [Game], platform: &str, id: &str) -> Option<&'a Game> {
    games.iter().find(|g| g.platform == platform && g.id == id)
}

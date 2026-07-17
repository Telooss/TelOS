//! La bibliothèque unifiée : agrège tous les providers de plateforme.
//! Miroir Rust de scripts/library.js. Aujourd'hui seul Steam est câblé,
//! mais chaque jeu porte déjà sa plateforme d'origine — ajouter un provider
//! revient à déposer un module dans platforms/ et à l'ajouter à scan_all().

use crate::platforms::{custom, steam};
use serde::Serialize;

#[derive(Serialize, Clone, Default)]
pub struct Art {
    pub portrait: Option<String>,
    pub hero: Option<String>,
    pub logo: Option<String>,
}

/// Comment démarrer un jeu — deux mécaniques, pas une.
///
/// `Uri` : un protocole résolu par l'OS (steam://, uplay://, origin2://…).
/// `Exec` : un exécutable lancé directement avec ses arguments — le cas des
/// émulateurs (`pcsx2.exe "rom.chd"`) et des .exe ajoutés à la main via le
/// bouton +. Poser les deux formes maintenant évite de refaire ce refactor
/// à travers tout le code quand l'émulation ou le bouton + arriveront.
#[derive(Clone, Debug)]
pub enum Launch {
    Uri(String),
    // Pas encore construite : arrive avec le bouton + et l'émulation
    // (chantiers suivants). Posée maintenant pour que launch_game() gère
    // déjà les deux mécaniques sans refactor à venir.
    #[allow(dead_code)]
    Exec { path: String, args: Vec<String> },
}

#[derive(Serialize, Clone)]
pub struct Game {
    pub platform: String,
    pub id: String,
    pub name: String,
    // Jamais envoyé au renderer : exposition minimale, et de toute façon
    // inutile côté client — c'est launch_game() qui l'exécute, côté natif.
    #[serde(skip)]
    pub launch: Launch,
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

/// Fait tourner un provider et range son résultat — factorisé pour ne pas
/// recopier ce bloc à chaque plateforme ajoutée (Epic, EA, GOG suivront
/// le même moule).
fn run_provider(
    id: &str,
    name: &str,
    present: bool,
    scan: impl FnOnce() -> Result<Vec<Game>, String>,
    games: &mut Vec<Game>,
    platforms: &mut Vec<PlatformStatus>,
) {
    if !present {
        platforms.push(PlatformStatus { id: id.into(), name: name.into(), present: false, count: 0, error: None });
        return;
    }
    match scan() {
        Ok(g) => {
            platforms.push(PlatformStatus { id: id.into(), name: name.into(), present: true, count: g.len(), error: None });
            games.extend(g);
        }
        Err(e) => platforms.push(PlatformStatus { id: id.into(), name: name.into(), present: true, count: 0, error: Some(e) }),
    }
}

/// Un provider en échec est signalé, jamais fatal aux autres :
/// une bibliothèque partielle vaut mieux qu'un écran vide.
pub fn scan_all() -> ScanResult {
    let mut games = Vec::new();
    let mut platforms = Vec::new();

    run_provider(steam::ID, steam::NAME, steam::detect(), steam::scan, &mut games, &mut platforms);
    run_provider(custom::ID, custom::NAME, custom::detect(), custom::scan, &mut games, &mut platforms);

    // Tri par récence, toutes plateformes confondues — loi n°4.
    games.sort_by(|a, b| b.last_played.cmp(&a.last_played));
    ScanResult { games, platforms }
}

pub fn find_game<'a>(games: &'a [Game], platform: &str, id: &str) -> Option<&'a Game> {
    games.iter().find(|g| g.platform == platform && g.id == id)
}

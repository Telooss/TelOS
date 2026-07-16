mod library;
mod platforms;
mod vdf;

use library::Game;
use std::sync::Mutex;

/// La bibliothèque scannée vit ici entre deux appels du renderer.
/// C'est ce qui permet à launch_game() de ne JAMAIS faire confiance à une
/// entrée brute du client : on ne relance que ce que NOTRE scan a trouvé.
struct AppState {
    games: Mutex<Vec<Game>>,
}

/// Un chemin disque brut -> data URI utilisable directement en `<img src>`.
/// Premier jet volontairement simple — à remplacer par le protocole
/// d'assets natif de Tauri une fois la tranche verticale validée.
fn encode_art(path: &str) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let mime = if path.to_lowercase().ends_with(".png") { "image/png" } else { "image/jpeg" };
    use base64::Engine;
    Some(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}

#[tauri::command]
fn get_games(state: tauri::State<AppState>) -> library::ScanResult {
    let t0 = std::time::Instant::now();
    let result = library::scan_all();
    eprintln!("[telos] scan_all : {} jeux en {:?}", result.games.len(), t0.elapsed());

    // L'état garde les CHEMINS bruts (utile pour plus tard), mais le renderer
    // reçoit déjà des data URI prêtes à afficher — pas de deuxième aller-retour par image.
    *state.games.lock().unwrap() = result.games.clone();

    let t1 = std::time::Instant::now();
    let games: Vec<Game> = result
        .games
        .into_iter()
        .map(|mut g| {
            g.art.portrait = g.art.portrait.as_deref().and_then(encode_art);
            g.art.hero = g.art.hero.as_deref().and_then(encode_art);
            g.art.logo = g.art.logo.as_deref().and_then(encode_art);
            g
        })
        .collect();

    let payload: usize = games
        .iter()
        .map(|g| {
            g.art.portrait.as_ref().map_or(0, |s| s.len())
                + g.art.hero.as_ref().map_or(0, |s| s.len())
                + g.art.logo.as_ref().map_or(0, |s| s.len())
        })
        .sum();
    eprintln!(
        "[telos] encodage jaquettes : {:?} — payload {:.1} Mo",
        t1.elapsed(),
        payload as f64 / 1e6
    );

    library::ScanResult { games, platforms: result.platforms }
}

/// Lance un jeu — mais UNIQUEMENT un jeu que NOTRE scan a trouvé.
/// Le renderer envoie (platform, id), jamais un chemin ni une URI :
/// c'est le cœur natif qui décide de ce qui est exécutable.
#[tauri::command]
fn launch_game(platform: String, id: String, state: tauri::State<AppState>) -> Result<(), String> {
    let games = state.games.lock().unwrap();
    let game = library::find_game(&games, &platform, &id)
        .ok_or_else(|| "Jeu introuvable dans la bibliothèque scannée.".to_string())?;
    open::that(&game.launch).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState { games: Mutex::new(Vec::new()) })
        .invoke_handler(tauri::generate_handler![get_games, launch_game])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod library;
mod platforms;
mod vdf;

use library::{Game, Launch};
use std::sync::Mutex;

/// La bibliothèque scannée vit ici entre deux appels du renderer.
/// C'est ce qui permet à launch_game() de ne JAMAIS faire confiance à une
/// entrée brute du client : on ne relance que ce que NOTRE scan a trouvé.
struct AppState {
    games: Mutex<Vec<Game>>,
}

/// L'IPC transporte des CHEMINS, jamais des octets. Le renderer convertit
/// chaque chemin en URL via convertFileSrc() (protocole d'assets natif de
/// Tauri) et le WebView charge l'image lui-même, depuis le disque, à la
/// demande — zéro copie à travers le pont IPC.
///
/// Avant : chaque jaquette était lue et encodée en base64 ICI, pour un
/// payload mesuré à 5.3 Mo sur 8 jeux. Ça grossit linéairement avec la
/// bibliothèque ; à l'échelle d'une logithèque d'émulation (~200 titres)
/// ça devenait ~130 Mo par appel. Le payload est désormais constant,
/// quel que soit le nombre de jeux.
#[tauri::command]
fn get_games(state: tauri::State<AppState>) -> library::ScanResult {
    let t0 = std::time::Instant::now();
    let result = library::scan_all();
    eprintln!("[telos] scan_all : {} jeux en {:?}", result.games.len(), t0.elapsed());

    *state.games.lock().unwrap() = result.games.clone();
    result
}

/// Bascule plein écran. En sans-bordure, il n'y a plus de croix de fermeture :
/// cette sortie doit exister AVANT d'activer le mode kiosque, sinon la fenêtre
/// devient un piège en développement.
#[tauri::command]
fn toggle_fullscreen(window: tauri::Window) -> Result<bool, String> {
    let now = window.is_fullscreen().map_err(|e| e.to_string())?;
    window.set_fullscreen(!now).map_err(|e| e.to_string())?;
    Ok(!now)
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

/// Lance un jeu — mais UNIQUEMENT un jeu que NOTRE scan a trouvé.
/// Le renderer envoie (platform, id), jamais un chemin, une URI ni des
/// arguments : c'est le cœur natif qui décide de ce qui est exécutable,
/// à partir de SA propre bibliothèque scannée.
#[tauri::command]
fn launch_game(platform: String, id: String, state: tauri::State<AppState>) -> Result<(), String> {
    let games = state.games.lock().unwrap();
    let game = library::find_game(&games, &platform, &id)
        .ok_or_else(|| "Jeu introuvable dans la bibliothèque scannée.".to_string())?;

    match &game.launch {
        Launch::Uri(uri) => open::that(uri).map_err(|e| e.to_string()),
        Launch::Exec { path, args } => {
            // spawn() et pas status()/output() : on ne bloque jamais sur la
            // durée d'une partie, et fermer telOS ne doit pas tuer le jeu.
            std::process::Command::new(path)
                .args(args)
                .spawn()
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState { games: Mutex::new(Vec::new()) })
        .invoke_handler(tauri::generate_handler![
            get_games,
            launch_game,
            toggle_fullscreen,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

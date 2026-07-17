mod art_downloader;
mod library;
mod platforms;
mod vdf;

use library::{Game, Launch};
use platforms::custom;
use std::sync::Mutex;
use tauri::{menu::{Menu, MenuItem}, tray::TrayIconBuilder, Manager, Emitter};
use tauri_plugin_dialog::DialogExt;

fn setup_gilrs(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mut gilrs = match gilrs::Gilrs::new() {
            Ok(g) => g,
            Err(_) => return,
        };
        loop {
            while let Some(gilrs::Event { event, .. }) = gilrs.next_event() {
                if let gilrs::EventType::ButtonPressed(gilrs::Button::Mode, _) = event {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = app.emit("wake-up", ());
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    });
}

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
fn get_games(app: tauri::AppHandle, state: tauri::State<AppState>) -> library::ScanResult {
    let t0 = std::time::Instant::now();
    let result = library::scan_all();
    eprintln!("[telos] scan_all : {} jeux en {:?}", result.games.len(), t0.elapsed());

    *state.games.lock().unwrap() = result.games.clone();

    // Lance le téléchargement des jaquettes manquantes en arrière-plan.
    // Le boot n'attend jamais le réseau (principe n°1 du plan perf).
    art_downloader::spawn_if_needed(app, result.games.clone());

    result
}

/// Ouvre le sélecteur natif pour choisir un exécutable. Bloquant : un
/// tauri::command tourne déjà hors du thread UI, donc attendre le choix de
/// l'utilisateur ici ne gèle rien côté rendu.
#[tauri::command]
fn pick_executable(app: tauri::AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .set_title("Choisir un exécutable")
        .add_filter("Exécutable", &["exe"])
        .blocking_pick_file()
        .map(|f| f.to_string())
}

/// Même mécanisme, sans filtre d'extension : le fichier optionnel passé en
/// argument (typiquement une ROM) n'a pas un format prévisible.
#[tauri::command]
fn pick_optional_file(app: tauri::AppHandle) -> Option<String> {
    app.dialog().file().set_title("Choisir un fichier (optionnel)").blocking_pick_file().map(|f| f.to_string())
}

/// Ajoute un jeu à la bibliothèque locale. Le chemin est revalidé sur disque
/// dans custom::add() même s'il vient du sélecteur natif : défense en
/// profondeur, jamais confiance aveugle en une chaîne venue du renderer.
/// Renvoie l'id du jeu créé — le renderer en a besoin pour le sélectionner
/// tout de suite après l'ajout, sans avoir à deviner sa position dans le rail.
#[tauri::command]
fn add_custom_game(
    name: String,
    platform: String,
    exec_path: String,
    args: Vec<String>,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let game = custom::add(name, platform, exec_path, args)?;
    let id = game.id.clone();
    state.games.lock().unwrap().push(game);
    Ok(id)
}

#[tauri::command]
fn remove_custom_game(id: String, state: tauri::State<AppState>) -> Result<(), String> {
    custom::remove(&id)?;
    state.games.lock().unwrap().retain(|g| g.id != id);
    Ok(())
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
fn launch_game(platform: String, id: String, state: tauri::State<AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let games = state.games.lock().unwrap();
    let game = library::find_game(&games, &platform, &id)
        .ok_or_else(|| "Jeu introuvable dans la bibliothèque scannée.".to_string())?;

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    let id_for_persist = game.id.clone();
    let result = match &game.launch {
        Launch::Uri(uri) => open::that(uri).map_err(|e| e.to_string()),
        Launch::Exec { path, args } => {
            let mut child = std::process::Command::new(path)
                .args(args)
                .spawn()
                .map_err(|e| e.to_string())?;
                
            let app_clone = app.clone();
            std::thread::spawn(move || {
                let _ = child.wait();
                if let Some(window) = app_clone.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = app_clone.emit("wake-up", ());
                }
            });
            Ok(())
        }
    };

    // Les jeux Steam tiennent leur lastPlayed de Steam lui-même (le manifeste).
    // Les jeux ajoutés à la main n'ont que nous — sans ça, un jeu lancé hier
    // resterait affiché comme "jamais lancé" pour toujours.
    if result.is_ok() && custom::is_custom_id(&id_for_persist) {
        drop(games); // relâche le verrou avant que mark_played rouvre le fichier
        custom::mark_played(&id_for_persist);
    }

    result
}

#[tauri::command]
fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let _ = window.hide();
                api.prevent_close();
            }
            _ => {}
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = app.emit("wake-up", ());
            }
        }))
        .setup(|app| {
            let show_i = MenuItem::with_id(app, "show", "Afficher telOS", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            TrayIconBuilder::with_id("telos-tray")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = app.emit("wake-up", ());
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = tray.app_handle().emit("wake-up", ());
                        }
                    }
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;

            setup_gilrs(app.handle().clone());
            // Si on lance avec --hidden (par exemple via un script au boot), on cache la fenêtre initiale
            if std::env::args().any(|a| a == "--hidden") {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            Ok(())
        })
        .manage(AppState { games: Mutex::new(Vec::new()) })
        .invoke_handler(tauri::generate_handler![
            get_games,
            launch_game,
            toggle_fullscreen,
            quit_app,
            hide_window,
            pick_executable,
            pick_optional_file,
            add_custom_game,
            remove_custom_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

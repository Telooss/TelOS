// Provider Steam.
//
// Contrat commun à toutes les plateformes (voir scripts/library.js) :
//   id, name  -> identité de la plateforme
//   scan()    -> { games: [...] }
// Chaque jeu retourné DOIT porter : platform, id, name, launch, art.
//
// Cette logique sera reportée côté Rust une fois le scaffold Tauri en place.

const ID = 'steam';
const NAME = 'Steam';

const fs = require('fs');
const path = require('path');
const os = require('os');

/**
 * Parse minimal du format VDF de Valve (KeyValues).
 * Le format est un arbre clé/valeur : "cle" "valeur" ou "cle" { ... }
 * On ne gère que ce dont on a besoin — pas de dépendance externe.
 */
function parseVDF(text) {
  const root = {};
  const stack = [root];
  // Capture soit une paire "cle" "valeur", soit "cle" seule (ouverture de bloc), soit { }
  const re = /"((?:[^"\\]|\\.)*)"\s*(?:"((?:[^"\\]|\\.)*)")?|(\{)|(\})/g;
  let m;
  let pendingKey = null;

  while ((m = re.exec(text)) !== null) {
    const [, key, value, open, close] = m;

    if (open) {
      const node = {};
      if (pendingKey !== null) {
        stack[stack.length - 1][pendingKey] = node;
        pendingKey = null;
      }
      stack.push(node);
    } else if (close) {
      if (stack.length > 1) stack.pop();
    } else if (key !== undefined) {
      if (value !== undefined) {
        stack[stack.length - 1][key] = value;
        pendingKey = null;
      } else {
        pendingKey = key;
      }
    }
  }
  return root;
}

/**
 * Tout ce que Steam installe mais qui n'est pas un jeu : runtimes, redistribuables, outils.
 * Ils ont un appmanifest exactement comme un jeu — seul un filtre explicite les distingue.
 */
const NON_GAME_APPIDS = new Set([
  '228980',  // Steamworks Common Redistributables
  '1070560', // Steam Linux Runtime
  '1391110', // Steam Linux Runtime - Soldier
  '1628350', // Steam Linux Runtime 3.0 (Sniper)
  '1493710', // Proton Experimental
  '2180100', // Proton Hotfix
]);
const NON_GAME_NAME = /redistributable|steam linux runtime|^proton\b|^steamworks/i;

function isGame(g) {
  return !NON_GAME_APPIDS.has(String(g.appid)) && !NON_GAME_NAME.test(g.name);
}

/** Trouve le dossier d'installation de Steam. */
function findSteamRoot() {
  const candidates = [
    path.join(process.env['ProgramFiles(x86)'] || 'C:\\Program Files (x86)', 'Steam'),
    path.join(process.env['ProgramFiles'] || 'C:\\Program Files', 'Steam'),
    'C:\\Steam',
    path.join(os.homedir(), '.steam', 'steam'), // Linux, pour plus tard
  ];
  return candidates.find(c => fs.existsSync(path.join(c, 'steamapps', 'libraryfolders.vdf'))) || null;
}

/** Liste tous les dossiers de bibliothèque (Steam peut en avoir sur plusieurs disques). */
function findLibraryFolders(steamRoot) {
  const vdfPath = path.join(steamRoot, 'steamapps', 'libraryfolders.vdf');
  const parsed = parseVDF(fs.readFileSync(vdfPath, 'utf8'));
  const folders = parsed.libraryfolders || parsed.LibraryFolders || {};

  const paths = [];
  for (const [key, val] of Object.entries(folders)) {
    if (!/^\d+$/.test(key)) continue; // les entrées sont indexées "0", "1", "2"...
    const p = typeof val === 'string' ? val : val.path;
    if (p) paths.push(p.replace(/\\\\/g, '\\'));
  }
  // Le dossier racine n'est pas toujours listé
  if (!paths.some(p => path.resolve(p) === path.resolve(steamRoot))) paths.unshift(steamRoot);
  return paths;
}

/** Lit les appmanifest_*.acf d'une bibliothèque et en sort les jeux installés. */
function readGames(libraryPath) {
  const appsDir = path.join(libraryPath, 'steamapps');
  if (!fs.existsSync(appsDir)) return [];

  return fs.readdirSync(appsDir)
    .filter(f => /^appmanifest_\d+\.acf$/.test(f))
    .map(f => {
      try {
        const state = parseVDF(fs.readFileSync(path.join(appsDir, f), 'utf8')).AppState;
        if (!state || !state.appid || !state.name) return null;
        return {
          platform: ID,                 // d'où vient le jeu — jamais implicite
          id: String(state.appid),      // identifiant DANS sa plateforme
          name: state.name,
          launch: `steam://rungameid/${state.appid}`,
          installDir: state.installdir,
          sizeBytes: Number(state.SizeOnDisk || 0),
          lastPlayed: Number(state.LastPlayed || 0), // epoch — notre tri par récence
          library: libraryPath,
        };
      } catch {
        return null; // un manifeste corrompu ne doit pas faire tomber tout le scan
      }
    })
    .filter(Boolean);
}

/** Cherche les jaquettes déjà présentes dans le cache local de Steam. */
function findArtwork(steamRoot, appid) {
  const cache = path.join(steamRoot, 'appcache', 'librarycache');
  const found = {};
  if (!fs.existsSync(cache)) return found;

  // Steam a changé de structure : parfois à plat, parfois dans un sous-dossier par appid
  const roots = [cache, path.join(cache, String(appid))];
  const wanted = { portrait: 'library_600x900', hero: 'library_hero', logo: 'logo' };

  for (const root of roots) {
    if (!fs.existsSync(root)) continue;
    let entries;
    try { entries = fs.readdirSync(root); } catch { continue; }
    for (const [kind, marker] of Object.entries(wanted)) {
      if (found[kind]) continue;
      const hit = entries.find(e =>
        e.toLowerCase().includes(marker) &&
        (root === cache ? e.startsWith(String(appid)) : true) &&
        /\.(jpg|png)$/i.test(e)
      );
      if (hit) found[kind] = path.join(root, hit);
    }
  }
  return found;
}

/** Détecte si la plateforme est présente sur la machine, sans lever d'erreur. */
function detect() {
  return findSteamRoot() !== null;
}

function scan() {
  const steamRoot = findSteamRoot();
  if (!steamRoot) throw new Error('Steam introuvable sur cette machine.');

  const libraries = findLibraryFolders(steamRoot);
  const games = libraries.flatMap(readGames).filter(isGame);
  for (const g of games) g.art = findArtwork(steamRoot, g.id);

  return { root: steamRoot, libraries, games };
}

module.exports = { id: ID, name: NAME, detect, scan, parseVDF, isGame };

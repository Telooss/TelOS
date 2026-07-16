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

/** Collecte récursivement les fichiers sous `dir`, jusqu'à `depth` niveaux. */
function collectFiles(dir, depth, out) {
  if (depth === 0) return;
  let entries;
  try { entries = fs.readdirSync(dir, { withFileTypes: true }); } catch { return; }
  for (const e of entries) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) collectFiles(p, depth - 1, out);
    else out.push(p);
  }
}

// `library_hero_blur.jpg` contient `library_hero` : sans exclusion explicite,
// on servirait la version floue à la place du vrai visuel.
const KIND = {
  portrait: f => f.includes('library_600x900'),
  hero: f => f.includes('library_hero') && !f.includes('library_hero_blur'),
  logo: f => f.includes('logo'),
};

/**
 * Steam a empilé trois structures de cache au fil des versions :
 *   librarycache/<appid>_library_hero.jpg          (à plat, ancien)
 *   librarycache/<appid>/library_hero.jpg          (2 niveaux)
 *   librarycache/<appid>/<hash>/library_hero.jpg   (3 niveaux, récent)
 * Les trois coexistent sur une même machine — d'où la recherche récursive.
 */
function findArtwork(steamRoot, appid) {
  const cache = path.join(steamRoot, 'appcache', 'librarycache');
  const found = {};
  if (!fs.existsSync(cache)) return found;

  const candidates = [];

  // Structures 2 et 3 niveaux : tout ce qui vit sous librarycache/<appid>/
  const appDir = path.join(cache, String(appid));
  if (fs.existsSync(appDir)) collectFiles(appDir, 3, candidates);

  // Structure à plat : librarycache/<appid>_library_hero.jpg
  try {
    const prefix = `${appid}_`;
    for (const e of fs.readdirSync(cache, { withFileTypes: true })) {
      if (e.isFile() && e.name.startsWith(prefix)) candidates.push(path.join(cache, e.name));
    }
  } catch { /* cache illisible : on repart avec ce qu'on a */ }

  for (const [kind, matches] of Object.entries(KIND)) {
    const hit = candidates.find(p => {
      const f = path.basename(p).toLowerCase();
      return /\.(jpg|png)$/.test(f) && matches(f);
    });
    if (hit) found[kind] = hit;
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

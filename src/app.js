// telOS — shell.
// Loi zéro : la nav répond dans la frame. Aucune animation ne fait attendre.
//
// Deux sources de données possibles, mêmes règles partout :
//   - navigateur (npm run dev)  -> fetch vers scripts/dev-server.js
//   - fenêtre Tauri             -> invoke des commandes Rust
// C'est la seule chose qui distingue les deux : tout le reste du shell
// (nav, rendu, séquence de lancement) est identique dans les deux mondes.
// Tauri v2 expose invoke à deux endroits selon la configuration. On teste les
// deux plutôt que de parier sur un seul : se tromper ici faisait silencieusement
// retomber sur fetch(), qui dans une fenêtre Tauri reste en attente pour
// toujours au lieu d'échouer — un blocage invisible, le pire des cas.
const tauriInvoke = () =>
  window.__TAURI__?.core?.invoke || window.__TAURI_INTERNALS__?.invoke || null;

async function apiGetGames() {
  const invoke = tauriInvoke();
  if (invoke) return invoke('get_games');

  // Filet de sécurité : un fetch qui ne répond pas doit échouer bruyamment,
  // jamais laisser l'écran figé sur son message de chargement.
  const ctrl = new AbortController();
  const timer = setTimeout(() => ctrl.abort(), 5000);
  try {
    const res = await fetch('/api/games', { signal: ctrl.signal });
    const data = await res.json();
    if (data.error) throw new Error(data.error);
    return data;
  } catch (e) {
    if (e.name === 'AbortError') throw new Error('Serveur injoignable (délai dépassé)');
    throw e;
  } finally {
    clearTimeout(timer);
  }
}

async function apiLaunch(g) {
  const invoke = tauriInvoke();
  if (invoke) return invoke('launch_game', { platform: g.platform, id: g.id });
  window.location.href = g.launch; // pas de retour possible en navigateur : on quitte la page
}

async function apiToggleFullscreen() {
  const invoke = tauriInvoke();
  if (invoke) return invoke('toggle_fullscreen');
}

async function apiQuit() {
  const invoke = tauriInvoke();
  if (invoke) return invoke('quit_app');
}

async function apiHideWindow() {
  const invoke = tauriInvoke();
  if (invoke) return invoke('hide_window');
}

// Le sélecteur de fichiers natif n'existe pas en navigateur (dev-server.js) :
// on échoue clairement plutôt que de laisser le bouton ne rien faire.
async function apiPickExecutable() {
  const invoke = tauriInvoke();
  if (!invoke) throw new Error('Sélecteur de fichiers indisponible hors app native');
  return invoke('pick_executable');
}
async function apiPickOptionalFile() {
  const invoke = tauriInvoke();
  if (!invoke) throw new Error('Sélecteur de fichiers indisponible hors app native');
  return invoke('pick_optional_file');
}
async function apiAddCustomGame(name, platform, execPath, args) {
  const invoke = tauriInvoke();
  if (!invoke) throw new Error('Ajout indisponible hors app native');
  return invoke('add_custom_game', { name, platform, execPath, args });
}

// Fiche jeu : contrairement aux jaquettes, l'attente réseau ici est normale —
// ouvrir la fiche est une action délibérée, pas le boot.
async function apiGetGameInfo(platform, id) {
  const invoke = tauriInvoke();
  if (!invoke) throw new Error('Fiche jeu indisponible hors app native');
  return invoke('get_game_info', { platform, id });
}
async function apiPickImage() {
  const invoke = tauriInvoke();
  if (!invoke) throw new Error('Sélecteur de fichiers indisponible hors app native');
  return invoke('pick_image');
}
async function apiSaveGameInfo(platform, id, patch) {
  const invoke = tauriInvoke();
  if (!invoke) throw new Error('Enregistrement indisponible hors app native');
  return invoke('save_game_info', { platform, id, patch });
}

/**
 * L'IPC ne transporte que des CHEMINS, jamais des octets (voir get_games
 * côté Rust). En Tauri, un chemin disque brut doit passer par convertFileSrc
 * pour devenir une URL chargeable par le WebView — celui-ci lit le fichier
 * lui-même, à la demande, sans repasser par le pont IPC.
 * En navigateur, dev-server.js sert déjà une URL HTTP utilisable telle quelle.
 */
function resolveArt(path) {
  if (!path) return null;
  const convert = window.__TAURI__?.core?.convertFileSrc;
  return convert ? convert(path) : path;
}

// Affichage de la plateforme — un seul endroit, valable pour les deux sources.
const PLATFORMS = {
  steam: { name: 'Steam', badge: 'STEAM' },
  epic: { name: 'Epic Games', badge: 'EPIC' },
  ea: { name: 'EA App', badge: 'EA' },
  ubisoft: { name: 'Ubisoft Connect', badge: 'UBI' },
  gog: { name: 'GOG', badge: 'GOG' },
  xbox: { name: 'Xbox / Game Pass', badge: 'XBOX' },
};
const platName = p => PLATFORMS[p]?.name || p;
const platBadge = p => PLATFORMS[p]?.badge || p.toUpperCase();

const rail = document.getElementById('rail');
const wall = document.getElementById('wall');
let games = [];
let index = 0;

const fmtSize = b => b ? (b / 1e9).toFixed(1) + ' GO' : '—';
const fmtDate = t => t
  ? new Date(t * 1000).toLocaleDateString('fr-FR', { day: '2-digit', month: 'short', year: 'numeric' }).toUpperCase()
  : 'JAMAIS LANCÉ';

/** Un écran figé sur son message de chargement ne dit rien. Une panne doit se voir. */
function fail(e) {
  document.getElementById('title').textContent = 'BIBLIOTHÈQUE INTROUVABLE';
  document.getElementById('meta').textContent = String(e && e.message ? e.message : e).toUpperCase();
  document.getElementById('led').style.background = 'var(--signal-red)';
  document.getElementById('host').textContent = 'HORS LIGNE';
  console.error('[telOS]', e);
}

// --- Séquence de Boot (DedSec Glitch) ---
const BOOT_LINES = [
  "INITIALIZING telOS KERNEL...",
  "BYPASSING SECURE BOOT...",
  "MOUNTING VIRTUAL FILESYSTEM...",
  "ESTABLISHING HANDSHAKE...",
  "DECRYPTING PAYLOAD...",
  "ACCESS GRANTED."
];

function playBootSequence() {
  const seq = document.getElementById('boot-seq');
  const term = document.getElementById('boot-term');
  const logo = seq.querySelector('.boot-logo');
  
  seq.classList.add('on');
  term.textContent = '';
  logo.classList.remove('flash');

  // Affichage des lignes du terminal plus lent
  BOOT_LINES.forEach((l, i) => {
    setTimeout(() => { term.textContent += l + '\n'; }, i * 180);
  });

  // À la fin des lignes, on attend un peu, on flash le logo
  const textEnd = BOOT_LINES.length * 180;
  
  setTimeout(() => {
    logo.classList.add('flash');
  }, textEnd + 200);

  // Et on cache l'overlay plus tard pour laisser le glitch visuel s'exprimer
  setTimeout(() => {
    seq.classList.remove('on');
  }, textEnd + 1200);
}

// Réveil depuis l'arrière-plan (ex: Moonlight s'y connecte, ou retour de jeu via manette)
window.__TAURI__?.event?.listen('wake-up', () => {
  playBootSequence();
  // On re-scan les jeux à chaque réveil pour toujours être à jour !
  boot().catch(fail);
});

async function boot() {
  const data = await apiGetGames();
  if (!data || !Array.isArray(data.games)) {
    throw new Error('Réponse inattendue de la bibliothèque');
  }

  games = data.games;
  document.getElementById('count').textContent = games.length;

  // render()/select() gèrent déjà une bibliothèque vide (la tuile + reste
  // sélectionnable) : pas de cas particulier à traiter ici.
  render();
  select(Math.min(index, games.length));
}

function render() {
  rail.innerHTML = '';
  for (const [i, g] of games.entries()) {
    const li = document.createElement('li');
    li.className = 'card';
    // Pas de jaquette en cache -> une affiche dans notre système, pas un trou gris
    const src = resolveArt(g.art.portrait);
    const cover = src
      ? `<img src="${src}" alt="" loading="lazy">`
      : `<div class="fallback">${g.name}</div>`;
    // La plateforme d'origine est toujours visible — jamais implicite.
    li.innerHTML = cover + `<span class="badge">${platBadge(g.platform)}</span>`;
    li.addEventListener('click', () => (i === index ? launch() : select(i)));
    rail.appendChild(li);
  }

  // La tuile + : jamais dans `games`, se comporte comme une carte de plus
  // (sélectionner puis activer) mais ouvre la modale au lieu de lancer.
  const addTile = document.createElement('li');
  addTile.className = 'card add';
  addTile.innerHTML = '<span class="plus">+</span>';
  addTile.addEventListener('click', () => (index === games.length ? launch() : select(games.length)));
  rail.appendChild(addTile);
}

let railX = 0;

function select(i) {
  // games.length est une position valide : c'est la tuile +, toujours en
  // dernier, jamais un jeu.
  const newIndex = Math.max(0, Math.min(games.length, i));
  if (newIndex !== index) audio.play('move');
  index = newIndex;
  
  const onAddTile = index === games.length;

  [...rail.children].forEach((c, k) => c.classList.toggle('sel', k === index));

  // Le rail reste calé à gauche et ne défile QUE si la sélection sortirait du cadre.
  // Centrer systématiquement laisserait la moitié de l'écran vide au premier jeu.
  const card = rail.children[index];
  const vpW = rail.parentElement.clientWidth;
  const left = card.offsetLeft;
  const right = left + card.offsetWidth;
  const M = 44; // marge de respiration
  if (left + railX < M) railX = M - left;
  if (right + railX > vpW - M) railX = vpW - M - right;
  railX = Math.min(0, railX);
  rail.style.transform = `translateX(${railX}px)`;

  if (onAddTile) {
    wall.style.backgroundImage = 'none';
    document.getElementById('kicker').textContent = 'BIBLIOTHÈQUE';
    document.getElementById('title').textContent = 'AJOUTER UN JEU';
    document.getElementById('meta').textContent = '.EXE HORS STEAM · ÉMULATEUR + ROM';
    return;
  }

  const g = games[index];

  // Le mur prend l'art du jeu — repli sur la jaquette si pas de hero,
  // sinon l'écran devient un trou noir mort.
  // Guillemets obligatoires : un chemin Windows non échappé dans url(...)
  // casse dès qu'il contient une parenthèse ("Program Files (x86)").
  const bg = resolveArt(g.art.hero || g.art.portrait);
  wall.style.backgroundImage = bg ? `url("${bg}")` : 'none';

  document.getElementById('kicker').textContent =
    `${platName(g.platform).toUpperCase()}  ·  ${g.lastPlayed ? 'CONTINUER' : 'JOUER'}`;
  document.getElementById('title').textContent = g.name;
  document.getElementById('meta').textContent = `${fmtSize(g.sizeBytes)}  ·  ${fmtDate(g.lastPlayed)}`;
}

// --- Le lancement : le seul endroit où la fiction s'exprime ---
const LINES = ['> ouverture du lien...', '> contournement...', '> injection charge utile'];

function launch() {
  if (index === games.length) { openAddModal(); return; }
  const g = games[index];
  const overlay = document.getElementById('intrusion');
  const log = document.getElementById('log');
  const grant = document.getElementById('grant');

  overlay.classList.add('on');
  log.textContent = '';
  grant.classList.remove('on');

  // Court et sec : ~1,2 s. Une intro qui fait attendre est un bug, pas du style.
  LINES.forEach((l, i) => setTimeout(() => (log.textContent += l + '\n'), i * 190));
  setTimeout(() => grant.classList.add('on'), 620);

  setTimeout(() => {
    // En Tauri : le cœur natif revérifie (platform, id) dans SA propre
    // bibliothèque scannée avant de lancer quoi que ce soit — jamais une
    // URI qui viendrait directement du renderer.
    apiLaunch(g);
  }, 900);

  setTimeout(() => overlay.classList.remove('on'), 1600);
}

// --- Bouton + : modale d'ajout (aucun réseau, aucun compte requis) ---
const modal = document.getElementById('add-modal');
const elName = document.getElementById('add-name');
const elPlatform = document.getElementById('add-platform');
const elExecBtn = document.getElementById('add-exec-btn');
const elExecPath = document.getElementById('add-exec-path');
const elArgBtn = document.getElementById('add-arg-btn');
const elArgPath = document.getElementById('add-arg-path');
const elError = document.getElementById('add-error');
const elCancel = document.getElementById('add-cancel');
const elSubmit = document.getElementById('add-submit');

// Tant que la modale est ouverte, TOUS les raccourcis globaux (nav du rail,
// A, F11, Échap-quitte) sont coupés — sinon une pression de manette derrière
// la modale relance un jeu ou ferme telOS pendant qu'on remplit un formulaire.
let modalOpen = false;

function updateSubmitState() {
  elSubmit.disabled = !(elName.value.trim() && pickedExec);
}

/** Dérive un nom lisible depuis un chemin : dossier retiré, extension retirée. */
function nameFromPath(p) {
  const base = p.split(/[\\/]/).pop() || p;
  return base.replace(/\.[^.]+$/, '');
}

let pickedExec = null;
let pickedArg = null;

function openAddModal() {
  modalOpen = true;
  elName.value = '';
  elPlatform.value = '';
  pickedExec = null;
  pickedArg = null;
  elExecPath.textContent = '— aucun —';
  elArgPath.textContent = '— aucun —';
  elError.textContent = '';
  updateSubmitState();
  modal.classList.add('on');
  elName.focus();
}

function closeAddModal() {
  modalOpen = false;
  modal.classList.remove('on');
}

elExecBtn.addEventListener('click', async () => {
  try {
    const path = await apiPickExecutable();
    if (!path) return; // annulé dans le sélecteur natif
    pickedExec = path;
    elExecPath.textContent = path;
    if (!elName.value.trim()) elName.value = nameFromPath(path);
    updateSubmitState();
  } catch (e) {
    elError.textContent = String(e.message || e);
  }
});

elArgBtn.addEventListener('click', async () => {
  try {
    const path = await apiPickOptionalFile();
    if (!path) return;
    pickedArg = path;
    elArgPath.textContent = path;
  } catch (e) {
    elError.textContent = String(e.message || e);
  }
});

elName.addEventListener('input', updateSubmitState);
elCancel.addEventListener('click', closeAddModal);

elSubmit.addEventListener('click', async () => {
  elError.textContent = '';
  try {
    const id = await apiAddCustomGame(
      elName.value.trim(),
      elPlatform.value.trim() || 'PC',
      pickedExec,
      pickedArg ? [pickedArg] : []
    );
    closeAddModal();
    await boot();
    select(Math.max(0, games.findIndex(g => g.id === id)));
  } catch (e) {
    elError.textContent = String(e.message || e);
  }
});

// --- Fiche jeu (touche Y) : info + édition, aucun impact sur le boot ---
const page = document.getElementById('game-page');
const pageWall = document.getElementById('page-wall');
const pageKicker = document.getElementById('page-kicker');
const pageTitle = document.getElementById('page-title');
const pageStats = document.getElementById('page-stats');
const pageChips = document.getElementById('page-chips');
const pageAge = document.getElementById('page-age');
const pageSummary = document.getElementById('page-summary');
const pageActions = document.getElementById('page-actions');
const pagePlayBtn = document.getElementById('page-play');
const pageEditForm = document.getElementById('page-edit-form');
const pageError = document.getElementById('page-error');
const pageBack = document.getElementById('page-back');
const pageEditBtn = document.getElementById('page-edit');
const pageCancelBtn = document.getElementById('page-cancel');
const pageSaveBtn = document.getElementById('page-save');
const editSummary = document.getElementById('edit-summary');
const editDeveloper = document.getElementById('edit-developer');
const editPublisher = document.getElementById('edit-publisher');
const editGenres = document.getElementById('edit-genres');
const editRelease = document.getElementById('edit-release');
const editPortraitBtn = document.getElementById('edit-portrait-btn');
const editPortraitPath = document.getElementById('edit-portrait-path');

// Même principe que modalOpen : tant que la fiche est ouverte, les
// raccourcis globaux du rail sont coupés.
let pageOpen = false;
let pageGame = null; // le jeu actuellement affiché — pour retrouver (platform, id) à la sauvegarde
let pickedPortrait = null;

/** Un stat block : label mono discret au-dessus d'une valeur nette. */
function statBlock(label, value) {
  const el = document.createElement('div');
  el.className = 'page-stat';
  el.innerHTML = `<span class="label">${label}</span><span class="value">${value}</span>`;
  return el;
}

/** Remplit les stats labellisées + les puces de genre. Rien à la place d'un
 * champ absent — pas de tiret ni de "N/A" qui alourdit l'écran pour rien. */
function renderPageStats(info, g) {
  pageStats.innerHTML = '';
  pageStats.appendChild(statBlock('Plateforme', platName(g.platform)));
  if (info.developer) pageStats.appendChild(statBlock('Développeur', info.developer));
  if (info.publisher && info.publisher !== info.developer) pageStats.appendChild(statBlock('Éditeur', info.publisher));
  if (info.releaseDate) pageStats.appendChild(statBlock('Sortie', info.releaseDate));
  // Metacritic existe côté Steam ET RAWG (même échelle /100). Le temps de
  // jeu, lui, ne vient que de RAWG — absent des métadonnées de store Steam,
  // ce n'est pas un trou, juste une donnée que cette source-là n'a pas.
  if (info.metacritic) pageStats.appendChild(statBlock('Metacritic', info.metacritic));
  if (info.playtimeHours) pageStats.appendChild(statBlock('Temps de jeu', `~${info.playtimeHours} H`));

  pageChips.innerHTML = (info.genres || []).map(g => `<span class="chip">${g}</span>`).join('');

  pageAge.hidden = !info.ageRating;
  pageAge.textContent = info.ageRating || '';
}

async function openGamePage(g) {
  pageOpen = true;
  pageGame = g;
  pickedPortrait = null;
  pageEditForm.hidden = true;
  pageActions.hidden = false;
  pageError.textContent = '';

  const bg = resolveArt(g.art.hero || g.art.portrait);
  pageWall.style.backgroundImage = bg ? `url("${bg}")` : 'none';
  pageKicker.textContent = 'FICHE';
  pageTitle.textContent = g.name;
  pageStats.innerHTML = '';
  pageChips.innerHTML = '';
  pageAge.hidden = true;
  pageSummary.textContent = 'Récupération des informations…';
  page.classList.add('on');

  // Le réseau peut prendre un instant ici — c'est une action délibérée de
  // l'utilisateur, pas le boot, donc l'attente est acceptable (contrairement
  // aux jaquettes, jamais sur le chemin critique).
  try {
    const info = await apiGetGameInfo(g.platform, g.id);
    if (!pageOpen || pageGame !== g) return; // la fiche a été fermée entre-temps
    pageGame.info = info; // mémorisé pour pré-remplir le formulaire d'édition
    renderPageStats(info, g);
    pageSummary.textContent = info.summary || 'Aucun résumé disponible pour ce jeu.';
  } catch (e) {
    pageSummary.textContent = 'Informations indisponibles : ' + String(e.message || e);
  }
}

// JOUER depuis la fiche : on ferme la page puis on déclenche EXACTEMENT le
// même lancement que depuis le rail — aucune logique dupliquée. pageGame
// correspond toujours à games[index], donc launch() cible le bon jeu.
pagePlayBtn.addEventListener('click', () => {
  closeGamePage();
  launch();
});

function closeGamePage() {
  pageOpen = false;
  page.classList.remove('on');
  pageEditForm.hidden = true;
  pageActions.hidden = false;
}

function openEditForm() {
  const info = pageGame?.info || {};
  editSummary.value = info.summary || '';
  editDeveloper.value = info.developer || '';
  editPublisher.value = info.publisher || '';
  editGenres.value = (info.genres || []).join(', ');
  editRelease.value = info.releaseDate || '';
  pickedPortrait = null;
  editPortraitPath.textContent = '— garder l\'actuelle —';
  pageError.textContent = '';
  pageActions.hidden = true;
  pageEditForm.hidden = false;
}

pageEditBtn.addEventListener('click', openEditForm);
pageCancelBtn.addEventListener('click', () => {
  pageEditForm.hidden = true;
  pageActions.hidden = false;
});
pageBack.addEventListener('click', closeGamePage);

editPortraitBtn.addEventListener('click', async () => {
  try {
    const path = await apiPickImage();
    if (!path) return;
    pickedPortrait = path;
    editPortraitPath.textContent = path;
  } catch (e) {
    pageError.textContent = String(e.message || e);
  }
});

pageSaveBtn.addEventListener('click', async () => {
  if (!pageGame) return;
  pageError.textContent = '';
  // Chaîne vide -> null : un champ qu'on n'a pas touché ne doit jamais
  // écraser une valeur déjà correcte côté cache ou surcharge précédente.
  const patch = {
    summary: editSummary.value.trim() || null,
    developer: editDeveloper.value.trim() || null,
    publisher: editPublisher.value.trim() || null,
    genres: editGenres.value.split(',').map(s => s.trim()).filter(Boolean),
    releaseDate: editRelease.value.trim() || null,
    portraitOverride: pickedPortrait || null,
  };
  try {
    await apiSaveGameInfo(pageGame.platform, pageGame.id, patch);
    pageEditForm.hidden = true;
    pageActions.hidden = false;
    // Une jaquette a pu changer : on refait tout le cycle get_games plutôt
    // qu'une mise à jour ciblée, exactement comme après un ajout au bouton +.
    await boot();
    await openGamePage(games.find(g => g.platform === pageGame.platform && g.id === pageGame.id) || pageGame);
  } catch (e) {
    pageError.textContent = String(e.message || e);
  }
});

// Écoute de la croix de fermeture pour réduire dans le System Tray
document.getElementById('btn-close')?.addEventListener('click', () => {
  apiHideWindow();
});

// --- Entrées : clavier, manette, tactile ---
addEventListener('keydown', e => {
  if (modalOpen) {
    // Un <input type="text"> gère déjà la saisie et Tab nativement — on ne
    // touche à rien d'autre pour ne pas casser ça. Seule Échap est à nous :
    // elle doit fermer la modale, pas quitter telOS.
    if (e.key === 'Escape') { e.preventDefault(); closeAddModal(); }
    return;
  }

  if (pageOpen) {
    // Même garde que la modale : la fiche a ses propres champs de saisie en
    // mode édition, Échap doit fermer LA FICHE, pas réduire toute l'app.
    if (e.key === 'Escape') { e.preventDefault(); closeGamePage(); }
    return;
  }

  // ÉCHAP pour réduire dans la barre des tâches
  if (e.key === 'Escape') { apiHideWindow(); return; }

  // La répétition auto de l'OS fait défiler la sélection en rafale et rend la
  // nav imprévisible : on n'accepte que des pressions discrètes.
  if (e.repeat) return;
  if (e.key === 'ArrowRight') select(index + 1);
  else if (e.key === 'ArrowLeft') select(index - 1);
  else if (e.key === 'Enter' || e.key === ' ') launch();
  // Y ouvre la fiche du jeu sélectionné — pas de fiche pour la tuile +.
  else if (e.key === 'y' || e.key === 'Y') { if (index < games.length) openGamePage(games[index]); }
  // Les deux sorties du mode kiosque : sans bordure, il n'y a plus de croix.
  else if (e.key === 'F11') apiToggleFullscreen();
  else if (e.key === 'Escape') apiQuit();
});

let padPrev = {};
(function pollPad() {
  for (const p of navigator.getGamepads?.() || []) {
    if (!p) continue;
    const ax = p.axes[0] || 0;
    const right = p.buttons[15]?.pressed || ax > 0.5;
    const left = p.buttons[14]?.pressed || ax < -0.5;
    const a = p.buttons[0]?.pressed;
    const b = p.buttons[1]?.pressed;
    const y = p.buttons[3]?.pressed; // mapping standard : 0=A 1=B 2=X 3=Y
    if (modalOpen) {
      if (b && !padPrev.b) closeAddModal(); // B annule, comme partout ailleurs
    } else if (pageOpen) {
      if (b && !padPrev.b) closeGamePage();
    } else {
      if (right && !padPrev.right) select(index + 1);
      if (left && !padPrev.left) select(index - 1);
      if (a && !padPrev.a) launch();
      if (y && !padPrev.y && index < games.length) openGamePage(games[index]);
    }
    padPrev = { right, left, a, b, y };
  }
  requestAnimationFrame(pollPad);
})();

let touchX = null;
addEventListener('touchstart', e => (touchX = e.touches[0].clientX), { passive: true });
addEventListener('touchend', e => {
  if (touchX === null || modalOpen || pageOpen) { touchX = null; return; }
  const dx = e.changedTouches[0].clientX - touchX;
  if (Math.abs(dx) > 45) select(index + (dx < 0 ? 1 : -1));
  touchX = null;
}, { passive: true });

// --- Horloge ---
(function tick() {
  document.getElementById('clock').textContent =
    new Date().toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' });
  setTimeout(tick, 10000);
})();

// --- Jaquettes en arrière-plan : écoute des téléchargements terminés ---
// Le cœur Rust télécharge les visuels manquants après le boot et émet un
// événement par jeu mis à jour. L'UI se rafraîchit sans recharger la page.
if (window.__TAURI__?.event) {
  window.__TAURI__.event.listen('game-art-ready', (event) => {
    const { platform, id, art } = event.payload;
    const game = games.find(g => g.platform === platform && g.id === id);
    if (!game) return;

    // Met à jour uniquement les visuels qui ont changé.
    if (art.portrait) game.art.portrait = art.portrait;
    if (art.hero) game.art.hero = art.hero;
    if (art.logo) game.art.logo = art.logo;

    // Rafraîchit la carte dans le rail.
    const gi = games.indexOf(game);
    if (gi >= 0 && gi < rail.children.length) {
      const src = resolveArt(game.art.portrait);
      const li = rail.children[gi];
      if (src) {
        const img = li.querySelector('img');
        const fallback = li.querySelector('.fallback');
        if (img) {
          img.src = src;
        } else if (fallback) {
          fallback.replaceWith(Object.assign(document.createElement('img'), { src, alt: '' }));
        }
      }
    }

    // Si ce jeu est sélectionné, rafraîchit aussi le mur de fond.
    if (gi === index) select(index);
  });
}

// select() gère seul le cas 0 jeu (la tuile + reste toujours positionnable) —
// plus besoin de conditionner sur games.length.
addEventListener('resize', () => select(index));

// Aucune erreur ne doit pouvoir passer inaperçue et laisser l'écran figé.
playBootSequence();
boot().catch(fail);

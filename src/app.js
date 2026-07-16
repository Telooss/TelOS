// telOS — shell.
// Loi zéro : la nav répond dans la frame. Aucune animation ne fait attendre.

const rail = document.getElementById('rail');
const wall = document.getElementById('wall');
let games = [];
let index = 0;

const fmtSize = b => b ? (b / 1e9).toFixed(1) + ' GO' : '—';
const fmtDate = t => t
  ? new Date(t * 1000).toLocaleDateString('fr-FR', { day: '2-digit', month: 'short', year: 'numeric' }).toUpperCase()
  : 'JAMAIS LANCÉ';

async function boot() {
  let data;
  try {
    const res = await fetch('/api/games');
    data = await res.json();
    if (data.error) throw new Error(data.error);
  } catch (e) {
    document.getElementById('title').textContent = 'BIBLIOTHÈQUE INTROUVABLE';
    document.getElementById('meta').textContent = String(e.message).toUpperCase();
    document.getElementById('led').style.background = 'var(--signal-red)';
    document.getElementById('host').textContent = 'HORS LIGNE';
    return;
  }

  games = data.games;
  document.getElementById('count').textContent = games.length;

  if (!games.length) {
    document.getElementById('title').textContent = 'AUCUN JEU INSTALLÉ';
    return;
  }

  render();
  select(0);
}

function render() {
  rail.innerHTML = '';
  for (const [i, g] of games.entries()) {
    const li = document.createElement('li');
    li.className = 'card';
    // Pas de jaquette en cache -> une affiche dans notre système, pas un trou gris
    const cover = g.art.portrait
      ? `<img src="${g.art.portrait}" alt="" loading="lazy">`
      : `<div class="fallback">${g.name}</div>`;
    // La plateforme d'origine est toujours visible — jamais implicite.
    li.innerHTML = cover + `<span class="badge">${g.platformBadge}</span>`;
    li.addEventListener('click', () => (i === index ? launch() : select(i)));
    rail.appendChild(li);
  }
}

let railX = 0;

function select(i) {
  index = Math.max(0, Math.min(games.length - 1, i));
  const g = games[index];

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

  // Le mur prend l'art du jeu — repli sur la jaquette si pas de hero,
  // sinon l'écran devient un trou noir mort.
  const bg = g.art.hero || g.art.portrait;
  wall.style.backgroundImage = bg ? `url(${bg})` : 'none';

  document.getElementById('kicker').textContent =
    `${g.platformName.toUpperCase()}  ·  ${g.lastPlayed ? 'CONTINUER' : 'JOUER'}`;
  document.getElementById('title').textContent = g.name;
  document.getElementById('meta').textContent = `${fmtSize(g.sizeBytes)}  ·  ${fmtDate(g.lastPlayed)}`;
}

// --- Le lancement : le seul endroit où la fiction s'exprime ---
const LINES = ['> ouverture du lien...', '> contournement...', '> injection charge utile'];

function launch() {
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
    // L'URI vient du provider de la plateforme, jamais codé en dur ici :
    // c'est ce qui permettra à Epic / EA / GOG de marcher sans toucher au shell.
    // Point de bascule vers Tauri : cette ligne deviendra une commande Rust
    // exposée explicitement, avec l'URI validé côté natif.
    window.location.href = g.launch;
  }, 900);

  setTimeout(() => overlay.classList.remove('on'), 1600);
}

// --- Entrées : clavier, manette, tactile ---
addEventListener('keydown', e => {
  // La répétition auto de l'OS fait défiler la sélection en rafale et rend la
  // nav imprévisible : on n'accepte que des pressions discrètes.
  if (e.repeat) return;
  if (e.key === 'ArrowRight') select(index + 1);
  else if (e.key === 'ArrowLeft') select(index - 1);
  else if (e.key === 'Enter' || e.key === ' ') launch();
});

let padPrev = {};
(function pollPad() {
  for (const p of navigator.getGamepads?.() || []) {
    if (!p) continue;
    const ax = p.axes[0] || 0;
    const right = p.buttons[15]?.pressed || ax > 0.5;
    const left = p.buttons[14]?.pressed || ax < -0.5;
    const a = p.buttons[0]?.pressed;
    if (right && !padPrev.right) select(index + 1);
    if (left && !padPrev.left) select(index - 1);
    if (a && !padPrev.a) launch();
    padPrev = { right, left, a };
  }
  requestAnimationFrame(pollPad);
})();

let touchX = null;
addEventListener('touchstart', e => (touchX = e.touches[0].clientX), { passive: true });
addEventListener('touchend', e => {
  if (touchX === null) return;
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

addEventListener('resize', () => games.length && select(index));
boot();

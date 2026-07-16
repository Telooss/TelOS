// La bibliothèque unifiée : agrège tous les providers de plateforme.
//
// Aujourd'hui seul Steam est implémenté, mais la couture existe dès maintenant :
// chaque jeu porte SA plateforme d'origine, et ajouter Epic / EA / GOG / Ubisoft
// revient à déposer un fichier dans platforms/ et à l'ajouter à la liste.
//
// Contrat d'un provider :
//   id       : identifiant court, stable ('steam')
//   name     : nom affichable ('Steam')
//   detect() : bool — la plateforme est-elle présente sur la machine ?
//   scan()   : { games: [...] } — chaque jeu porte platform, id, name, launch, art
//
// Un provider qui échoue ne doit JAMAIS faire tomber les autres : une bibliothèque
// partielle vaut mieux qu'un écran vide.

const steam = require('./platforms/steam');

// À venir : epic, ea, ubisoft, gog. Voir docs/roadmap.md.
const PROVIDERS = [steam];

/** Métadonnées d'affichage par plateforme. Le badge est volontairement court. */
const PLATFORMS = {
  steam:   { name: 'Steam',           badge: 'STEAM' },
  epic:    { name: 'Epic Games',      badge: 'EPIC' },
  ea:      { name: 'EA App',          badge: 'EA' },
  ubisoft: { name: 'Ubisoft Connect', badge: 'UBI' },
  gog:     { name: 'GOG',             badge: 'GOG' },
  xbox:    { name: 'Xbox / Game Pass', badge: 'XBOX' },
};

function scanAll() {
  const games = [];
  const platforms = [];

  for (const p of PROVIDERS) {
    if (!p.detect()) {
      platforms.push({ id: p.id, name: p.name, present: false, count: 0 });
      continue;
    }
    try {
      const res = p.scan();
      games.push(...res.games);
      platforms.push({ id: p.id, name: p.name, present: true, count: res.games.length });
    } catch (e) {
      // Un provider cassé est signalé, pas fatal.
      platforms.push({ id: p.id, name: p.name, present: true, count: 0, error: e.message });
    }
  }

  // Tri par récence, toutes plateformes confondues — loi n°4.
  games.sort((a, b) => b.lastPlayed - a.lastPlayed);
  return { games, platforms };
}

/** Retrouve un jeu par plateforme + id. Utilisé pour servir les jaquettes. */
function findGame(games, platform, id) {
  return games.find(g => g.platform === platform && String(g.id) === String(id)) || null;
}

// --- Sortie lisible : node scripts/library.js ---
if (require.main === module) {
  const { games, platforms } = scanAll();

  for (const p of platforms) {
    const state = !p.present ? 'absente' : p.error ? `ERREUR: ${p.error}` : `${p.count} jeux`;
    console.log(`${p.name.padEnd(18)} ${state}`);
  }
  console.log(`\n${games.length} jeux au total\n`);

  const size = n => (n ? (n / 1e9).toFixed(1) + ' Go' : '—');
  const date = t => (t ? new Date(t * 1000).toLocaleDateString('fr-FR') : '—');

  for (const g of games) {
    const art = ['portrait', 'hero', 'logo'].filter(k => g.art[k]).join('+') || 'AUCUN VISUEL';
    console.log(
      `${(PLATFORMS[g.platform]?.badge || g.platform).padEnd(6)} ${String(g.id).padStart(8)}  ` +
      `${g.name.slice(0, 34).padEnd(34)} ${size(g.sizeBytes).padStart(8)}  ${date(g.lastPlayed).padStart(10)}  ${art}`
    );
  }
}

module.exports = { scanAll, findGame, PLATFORMS };

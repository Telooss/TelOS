// Serveur de dev pour l'UI telOS (src/).
// Zéro dépendance. Tauri servira src/ de la même façon, donc rien à jeter plus tard.
//
// Expose aussi la vraie bibliothèque Steam :
//   GET /api/games        -> JSON des jeux installés
//   GET /art/<appid>/<kind> -> jaquette depuis le cache local de Steam
//
// Ces deux routes sont l'équivalent temporaire des futures commandes Tauri.
const http = require('http');
const fs = require('fs');
const path = require('path');
const { scan } = require('./scan-steam');

const PORT = process.env.PORT || 5173;
const ROOT = path.join(__dirname, '..', 'src');

const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.webp': 'image/webp',
  '.woff2': 'font/woff2',
  '.ttf': 'font/ttf',
  '.otf': 'font/otf',
};

// La bibliothèque est scannée une fois au démarrage puis mise en cache :
// relire des dizaines d'appmanifest à chaque requête serait absurde.
let library = { games: [], error: null };
function refresh() {
  try {
    library = { ...scan(), error: null };
    console.log(`Bibliothèque : ${library.games.length} jeux`);
  } catch (e) {
    library = { games: [], error: e.message };
    console.error('Scan Steam impossible :', e.message);
  }
}

function sendJSON(res, code, data) {
  const body = JSON.stringify(data);
  res.writeHead(code, { 'Content-Type': MIME['.json'], 'Cache-Control': 'no-store' });
  res.end(body);
}

http.createServer((req, res) => {
  const url = decodeURIComponent(req.url.split('?')[0]);

  // --- API : la bibliothèque ---
  if (url === '/api/games') {
    if (library.error) return sendJSON(res, 500, { error: library.error });
    // On n'expose pas les chemins disque au client : juste ce qu'il faut pour afficher.
    return sendJSON(res, 200, {
      games: library.games.map(g => ({
        appid: g.appid,
        name: g.name,
        sizeBytes: g.sizeBytes,
        lastPlayed: g.lastPlayed,
        art: {
          portrait: g.art.portrait ? `/art/${g.appid}/portrait` : null,
          hero: g.art.hero ? `/art/${g.appid}/hero` : null,
          logo: g.art.logo ? `/art/${g.appid}/logo` : null,
        },
      })),
    });
  }

  // --- Jaquettes : servies depuis le cache Steam ---
  const art = url.match(/^\/art\/(\d+)\/(portrait|hero|logo)$/);
  if (art) {
    const [, appid, kind] = art;
    // Le chemin vient de NOTRE scan, jamais de l'entrée client : pas de traversal possible.
    const game = library.games.find(g => String(g.appid) === appid);
    const file = game && game.art[kind];
    if (!file || !fs.existsSync(file)) { res.writeHead(404); return res.end('no art'); }
    return fs.readFile(file, (err, data) => {
      if (err) { res.writeHead(404); return res.end('no art'); }
      res.writeHead(200, {
        'Content-Type': MIME[path.extname(file).toLowerCase()] || 'image/jpeg',
        'Cache-Control': 'max-age=3600',
      });
      res.end(data);
    });
  }

  // --- Fichiers statiques ---
  const file = path.join(ROOT, path.normalize(url === '/' ? '/index.html' : url));
  if (!file.startsWith(ROOT)) { res.writeHead(403); return res.end('Forbidden'); }

  fs.readFile(file, (err, data) => {
    if (err) {
      res.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' });
      return res.end('404 — ' + url);
    }
    res.writeHead(200, {
      'Content-Type': MIME[path.extname(file).toLowerCase()] || 'application/octet-stream',
      'Cache-Control': 'no-store', // on itère sur le design : jamais de cache
    });
    res.end(data);
  });
}).listen(PORT, '127.0.0.1', () => {
  refresh();
  console.log(`telOS dev → http://localhost:${PORT}`);
});

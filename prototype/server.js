// Serveur statique minimal, zéro dépendance.
// Sert le dossier courant pour que le téléphone puisse ouvrir l'UI via le Wi-Fi.
const http = require('http');
const fs = require('fs');
const path = require('path');
const os = require('os');

const PORT = process.env.PORT || 5173;
const ROOT = __dirname;
const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg',
  '.webp': 'image/webp', '.svg': 'image/svg+xml', '.ico': 'image/x-icon',
  '.woff2': 'font/woff2',
};

const server = http.createServer((req, res) => {
  let url = decodeURIComponent(req.url.split('?')[0]);
  if (url === '/') url = '/index.html';
  const file = path.join(ROOT, path.normalize(url));
  if (!file.startsWith(ROOT)) { res.writeHead(403); return res.end('Forbidden'); }
  fs.readFile(file, (err, data) => {
    if (err) { res.writeHead(404); return res.end('Not found'); }
    res.writeHead(200, { 'Content-Type': MIME[path.extname(file).toLowerCase()] || 'application/octet-stream' });
    res.end(data);
  });
});

server.listen(PORT, '0.0.0.0', () => {
  const ips = Object.values(os.networkInterfaces()).flat()
    .filter(i => i && i.family === 'IPv4' && !i.internal).map(i => i.address);
  console.log('\n  NOVA lancé.\n');
  console.log('  Sur ce PC     : http://localhost:' + PORT);
  ips.forEach(ip => console.log('  Sur ton tel   : http://' + ip + ':' + PORT + '   (même Wi-Fi)'));
  console.log('\n  Ctrl+C pour arrêter.\n');
});

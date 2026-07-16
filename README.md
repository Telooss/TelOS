# telOS

> Un shell console plein écran qui remplace Steam Big Picture — pensé pour être **streamé**, pas pour être regardé de près.

*(Le nom s'écrit `telOS` — `tel` en minuscules, `OS` en capitales — un clin d'œil à ctOS dans Watch Dogs. Le repo GitHub et le paquet npm restent `TelOS` / `telos`, conventions techniques normales.)*

## L'idée

telOS tourne **sur le PC hôte**. [Sunshine](https://github.com/LizardByte/Sunshine) streame son affichage, et [Moonlight](https://moonlight-stream.org) le reçoit sur un handheld / téléphone / tablette. Le client distant n'est **qu'un écran et une manette** : tout — l'UI, la bibliothèque, le lancement des jeux — s'exécute sur l'hôte.

```
┌──────────────── PC hôte (Windows) ────────────────┐        ┌──── Client ────┐
│                                                   │        │                │
│   TelOS (plein écran)  ──lance──>  Steam / .exe   │        │                │
│        │                                          │        │                │
│        └── affichage ──> Sunshine ──── H.265/AV1 ─┼───────>│   Moonlight    │
│                              <──── manette/input ─┼────────┤                │
└───────────────────────────────────────────────────┘        └────────────────┘
```

**Pourquoi ça compte :** parce que l'UI n'est pas une app sur le téléphone, le lancement des jeux est **local et natif** (`steam://rungameid/…`, spawn de process) au lieu de dépendre de deep-links Android. Le client reste bête et interchangeable.

## Objectif d'usage

Jouer confortablement à ses titres PC depuis un handheld, y compris **via un partage de connexion mobile** — sans passer par une interface générique et lourde.

## État

🚧 **Pré-alpha.** Architecture et identité visuelle arrêtées. Rien de fonctionnel n'est encore livré.

👉 Plan de réalisation phasé complet (design, natif, streaming, et les extensions musique / communication / multi-client) : voir [`docs/roadmap.md`](docs/roadmap.md) — miroir du plan tenu dans Notion.

| Chantier | État |
|---|---|
| Architecture (hôte vs client) | ✅ Arrêtée |
| Choix du moteur | ✅ Tauri (voir ci-dessous) |
| Recherche design | ✅ [`docs/design-research.md`](docs/design-research.md) |
| Direction artistique (DedSec / WD2) | ✅ Arrêtée |
| Système typographique (`telOS`, Oxanium + Doctor Glitch) | ✅ Arrêté |
| Maquette du shell | 🔄 En cours |
| Cœur natif Tauri | ⬜ À faire |
| Scan bibliothèque Steam | ⬜ À faire |
| Lancement des jeux | ⬜ À faire |
| Intégration Sunshine | ⬜ À faire |
| Musique / communication / multi-client | ⬜ À faire — voir roadmap |

## Décisions techniques

### Moteur : Tauri (et pas Electron)

Deux exigences dirigent le projet : **optimisation** et **bonnes pratiques de sécurité**. Tauri gagne sur les deux.

- **Empreinte** : utilise le WebView système (WebView2, préinstallé sur Windows 11) au lieu d'embarquer un Chromium complet. ~50 Mo contre ~150+ Mo.
- **Sécurité par défaut** : le renderer est sandboxé et ne peut atteindre le système **que** via des commandes exposées explicitement, une par une (lister la bibliothèque, lancer une cible). Cœur en Rust, CSP active d'office. Avec Electron, la même posture demande un durcissement manuel (`contextIsolation`, `nodeIntegration: false`, sandbox, validation IPC) où chaque oubli est une faille.
- **Contrepartie assumée** : nécessite le toolchain Rust.

L'UI étant du web standard, elle reste **portable** : le design se cale avant que le moteur soit installé.

### Principes de sécurité

- Aucune commande native n'est exposée au renderer sans nécessité explicite.
- Aucun chemin d'exécutable ni argument ne provient d'une entrée non validée.
- La bibliothèque scannée et les caches de jaquettes sont des **données locales** et ne sont jamais committés.

## Design

Le design ne part pas d'un template. Il part d'une **analyse des interfaces console existantes** (PS4, PS5, Xbox 360 / One / Series X, DS, Switch, Wii), dont on extrait des lois réutilisables — puis d'une direction artistique assumée.

La thèse actuelle : **squelette de Switch, peau de DedSec.** Structure rapide et ascétique en dessous, identité punk/hacktiviste en surface.

👉 Tout est documenté dans [`docs/design-research.md`](docs/design-research.md).

### Contrainte structurante

L'interface est **compressée en H.265 et affichée sur un petit écran, parfois à 10 Mbps sur réseau mobile**. Ça exclut le texte fin, les dégradés subtils (banding atroce en streaming) et les détails de 1 px. Ça impose des aplats francs, un contraste élevé, une typo grasse et de gros objets. Cette contrainte n'est pas une gêne : c'est un **cadre esthétique**.

## Structure

```
docs/                    Recherche design, décisions, roadmap
src/                      L'UI de telOS (HTML/CSS/JS), servie telle quelle par Tauri plus tard
src/fonts/                Oxanium (OFL, committée) ; Doctor Glitch (usage perso, gitignorée — voir docs/design-research.md)
scripts/dev-server.js     Serveur de dev, sert src/
.claude/skills/telos-design/  Skill projet : système de design, interdits, boucle de vérification visuelle
prototype/                Exploration jetable (première itération web, abandonnée)
```

## Développement

```bash
npm run dev         # sert src/ sur http://localhost:5173
```

## Prototype (historique)

Le dossier `prototype/` contient la toute première exploration, faite avant que l'architecture soit clarifiée. **Elle ne reflète plus la direction du projet** et n'est conservée qu'à titre de trace.

```bash
npm run prototype   # sert prototype/ sur http://localhost:5173
```

## Licence

Projet personnel, tous droits réservés pour l'instant.

# Plan — Bibliothèque universelle & jaquettes

> Objectif : couvrir **tous** les jeux d'Alan — Steam, autres launchers, `.exe` maison, **et ROMs émulées** — sans jamais dégrader les performances du shell.

## La contrainte qui commande tout

telOS est un **shell de console streamé**. Il doit répondre dans la frame (loi zéro). Chaque ajout ci-dessous touche à deux choses dangereuses pour ça : **le réseau** et **les images**. D'où cinq principes non négociables.

### Les 5 principes de performance

1. **Le réseau n'est JAMAIS sur le chemin critique.** L'écran s'affiche avec ce qui est en local. L'enrichissement arrive après, et met à jour l'UI quand il arrive. Un boot ne doit jamais attendre une réponse HTTP.
2. **Une jaquette téléchargée l'est une fois pour toutes.** Cache disque permanent — une box art ne change pas. Deuxième lancement = zéro réseau.
3. **Cache négatif obligatoire.** Si une source ne trouve rien, on l'écrit. Sans ça, on interroge l'API à chaque boot pour un jeu qui n'aura jamais d'art. C'est le piège classique.
4. **L'IPC transporte des chemins, jamais des octets.** Le WebView charge les images lui-même via le protocole d'assets.
5. **Chargement paresseux.** Seules les jaquettes visibles sont décodées.

---

## Étape 0 — Le pipeline de jaquettes *(prérequis, à faire en premier)*

**Pourquoi d'abord :** `get_games` encode aujourd'hui toutes les jaquettes en base64 dans **un seul message IPC**. Mesuré : **5,3 Mo pour 8 jeux**. Ça croît linéairement — une logithèque d'émulation de 200 titres donnerait **~130 Mo par appel**. Ajouter des sources réseau avant de corriger ça, c'est bâtir sur une fondation connue comme mauvaise.

**Ce qu'on fait**
- Activer `app.security.assetProtocol` dans `tauri.conf.json`, avec un **scope restreint** aux seuls dossiers légitimes (cache Steam, notre cache) — pas d'accès disque large.
- Ajouter `asset:` et `http://asset.localhost` à la CSP (`img-src`).
- Côté shell : `convertFileSrc(chemin)` au lieu des data URI.
- Supprimer `encode_art()` et la dépendance `base64`.

**Cible mesurable :** payload IPC **5,3 Mo → < 50 Ko**, et **constant** quel que soit le nombre de jeux.

---

## Étape 1 — Le modèle de lancement *(fondation commune)*

**Pourquoi maintenant :** `launch` est aujourd'hui une simple `String` (une URI) passée à `open::that()`. Un émulateur, c'est `pcsx2.exe "rom.chd"` — un exécutable **avec des arguments**. Deux mécaniques différentes. Le bouton `+` a exactement le même besoin. Si on ne le pose pas maintenant, c'est un refactor à travers tout le code plus tard — la même erreur qu'on a évitée avec la couture multi-plateforme.

```rust
enum Launch {
    Uri(String),                               // steam://, uplay://, origin2://
    Exec { path: PathBuf, args: Vec<String> }, // émulateur + ROM, .exe maison
}
```

**Sécurité (inchangée, c'est la règle du projet)** : le renderer envoie toujours `(platform, id)`. Le cœur natif résout la cible dans **sa propre** bibliothèque scannée. Aucun chemin, aucune URI, aucun argument ne vient jamais du client.

**Coût perf :** nul.

---

## Étape 2 — Le bouton `+` *(aucun compte, aucun réseau)*

- Sélecteur de fichiers **natif** (`tauri-plugin-dialog`).
- Le jeu ajouté est stocké dans une config locale (`%APPDATA%/telOS/`), gitignorée.
- Il devient un **provider comme les autres** (`platform: "custom"`), donc badge, tri par récence et lancement marchent gratuitement.
- Couvre aussi l'émulation : on choisit l'émulateur comme exécutable, la ROM comme argument.

**Coût perf :** nul — local, pas de réseau.

---

## Étape 3 — CDN Steam *(gratuit, sans clé, sans compte)*

Règle les jaquettes manquantes des jeux **Steam** uniquement (il faut un appid).

- URL publique : `https://cdn.cloudflare.steamstatic.com/steam/apps/<appid>/library_600x900.jpg`
- **Prouvé** : testé sur Megabonk (appid 3405340) → HTTP 200, 152 Ko.
- Déclenché **en arrière-plan après l'affichage**, jamais au boot.
- Téléchargé une fois → cache disque → plus jamais.
- Cache négatif si 404.

**Vie privée :** requête anonyme vers un CDN public, aucun compte, aucun identifiant. Steam apprend qu'une IP a demandé l'image d'un jeu — rien de plus.

---

## Étape 4 — libretro-thumbnails *(gratuit, sans clé, rétro)*

Box art de ROMs depuis un dépôt GitHub public.

- **Testé** : Chrono Trigger (SNES) → HTTP 200, 275 Ko ✅
- **Limite trouvée au test** : Metal Gear Solid (PS1) → 26 octets, pas d'image ❌. La source exige que le nom colle **exactement** à la convention No-Intro/Redump. Gratuit mais rigide.
- Utile en première tentative pour les ROMs bien nommés ; SteamGridDB rattrape le reste.

---

## Étape 5 — SteamGridDB *(clé requise — le seul compte)*

Le filet universel : Steam, hors Steam, **et rétro** (c'est la base qu'utilise la communauté émulation).

- Recherche **floue par nom** → tolérante aux noms approximatifs, contrairement à libretro-thumbnails.
- Clé dans la config locale, **gitignorée**, jamais committée (même règle que Doctor Glitch).
- Appelée **uniquement** pour ce que les étapes 3 et 4 n'ont pas couvert.
- Sur la bibliothèque actuelle d'Alan (8 jeux Steam), elle ne serait **jamais** appelée. C'est un filet, pas une dépendance.

---

## Budget de performance

| Métrique | Cible |
|---|---|
| Boot → interactif | **< 100 ms** |
| Payload IPC | **< 100 Ko**, constant quel que soit le nombre de jeux |
| Appels réseau au boot (cache chaud) | **0** |
| Scan bibliothèque | < 50 ms (mesuré : 10-46 ms pour 8 jeux) |

## Ordre d'exécution

```
0. Pipeline jaquettes   ← prérequis perf, débloque tout le reste
1. Modèle de lancement  ← fondation commune (+ et émulation)
2. Bouton +             ← aucun compte
3. CDN Steam            ← aucun compte
4. libretro-thumbnails  ← aucun compte
5. SteamGridDB          ← la clé d'Alan
```

Les cinq premières étapes ne demandent **rien** à Alan. Seule la dernière attend sa clé.

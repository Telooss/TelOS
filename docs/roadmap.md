# Plan de réalisation

> Miroir du plan tenu dans Notion (page « Plan de réalisation » + base « Chantiers telOS »). Le scope a grossi : ce n'est plus « un launcher », c'est un vrai petit système. Ce document découpe le travail en phases logiques, avec pour chaque idée nouvelle une évaluation honnête de faisabilité — pas de promesse en l'air.

---

## Phase 0 — Fondations ✅ Arrêtée

- Architecture hôte/client : telOS tourne sur le PC, Moonlight n'est qu'écran + manette
- Moteur : Tauri (WebView2, sandboxé, commandes explicites)
- Recherche design : 8 UI console analysées, identité DedSec/WD2 décomposée
- Repo GitHub + Notion posés

## Phase 1 — Identité visuelle ✅ Arrêtée

- Ancrage **DedSec / Watch Dogs 2** tranché (coloré, béton + bombe — pas Legion)
- Métaphore d'intrusion : tranchée, **vit uniquement dans la séquence de lancement**, jamais dans l'UI au repos
- Système typographique : **Oxanium** (interface) + **Doctor Glitch** (identité/événements, usage perso, jamais committée)
- Wordmark `telOS` : composite « tel » discret + « OS » glitché, avec un Ø dessiné à la main
- Skill projet `telos-design` écrite : lois structurelles, tokens, interdits, checklist de critique — chargée à chaque session
- ⬜ Logo SVG final à valider visuellement (bloqué temporairement par un souci d'outil de capture d'écran côté environnement de dev, pas un problème de conception)

Détails complets : [`design-research.md`](design-research.md).

## Phase 2 — Maquette du shell haute-fidélité 🔄 En cours

L'écran principal, en vrai, dans le navigateur, avec la boucle écrire → regarder → corriger :

- Le mur (art du jeu sélectionné, plein cadre) + hero + rail de récence
- Bandeau de statut ambiant (leçon DS) — avec de vraies données dès que possible, pas des placeholders
- Barre système fine (leçon Switch)
- Overlay Control Center (leçon PS5/Xbox) — changer de jeu / couper le stream sans tout quitter
- Séquence de lancement — l'unique moment où le glitch/l'intrusion s'exprime
- Tester les états limites : bibliothèque vide, 1 seul jeu, titre très long

## Phase 3 — Cœur natif (Tauri)

Commande de scaffold retenue (Windows / PowerShell) :

```powershell
irm https://create.tauri.app/ps | iex
```

*(Équivalent npm si besoin : `npm create tauri-app@latest`. La variante `sh <(curl …)` de la doc est la voie Linux — inadaptée ici.)*

- Installer le toolchain Rust, scaffold Tauri pointant sur `src/`
- CSP stricte, permissions minimales, aucune commande exposée sans nécessité explicite
- Persistance locale (config utilisateur, cache de bibliothèque) — jamais committée
- Packaging Windows

## Phase 4 — Bibliothèque & lancement

- Parser la bibliothèque Steam (`libraryfolders.vdf`, `appmanifest_*.acf`)
- Récupérer les vraies jaquettes (cache Steam local, ou SteamGridDB en secours)
- Lancer un jeu : `steam://rungameid/<appid>`, spawn de process pour les exécutables hors Steam
- Toute entrée (chemin, arguments) validée — aucune exécution à partir d'une donnée non contrôlée

## Phase 5 — Intégration Sunshine / Moonlight

- Installer et configurer Sunshine sur l'hôte
- Déclarer **telOS comme application Sunshine dédiée** (pas juste « Desktop ») : Moonlight doit tomber directement dessus
- Test LAN de bout en bout (cible : latence réseau < 5 ms, 0 perte)
- Le bandeau de statut affiche les **vraies** stats (latence, débit, hôte) — c'est ce qui rend la fiction honnête plutôt que décorative

## Phase 6 — Accès distant

- Le partage de connexion mobile est derrière du CGNAT : pas de redirection de port possible
- **Tailscale** pour traverser proprement (vérifier connexion directe vs relais DERP)
- Bitrate adaptatif selon le lien, attention à la conso data (~9 Go/h à 20 Mbps)

## Phase 7 — Extensions salon *(nouveau scope)*

### 🎵 Musique — faisable, chemin clair

- **Spotify Web API** : OAuth, pochette + titre en cours, contrôle lecture/pause/skip. Officiel, bien documenté. Le contrôle de lecture nécessite Spotify Premium côté utilisateur.
- Alternative locale à étudier plus tard si besoin (fichiers locaux, pont vers un lecteur déjà présent sur l'hôte)

### 💬 Discord — faisable, mais pas comme une réimplémentation le suggérerait

**Il n'existe pas d'API officielle permettant à une app tierce d'embarquer un appel vocal Discord en tant qu'utilisateur.** La seule voie non-officielle (automatiser le client avec un token utilisateur, dit « self-bot ») **viole les CGU Discord** et met le compte en danger — exclue d'office.

**La bonne nouvelle : telOS tourne sur le PC hôte, donc pas besoin de réimplémenter Discord.** On peut simplement afficher/superposer le **vrai client Discord** (fenêtre native, en overlay ou en tuile) — le vocal fonctionne nativement, zéro reverse engineering, zéro risque.

En complément, légitime et bien documenté : le **SDK officiel Discord Rich Presence**, pour afficher « en train de jouer à X » depuis telOS.

### 📺 Multi-client (Fire TV Stick et au-delà)

**Aucune nouvelle architecture requise.** L'architecture hôte/client posée en Phase 0 tenait déjà compte de ça — Moonlight est juste un client parmi d'autres, que ce soit un handheld, un téléphone ou une TV.

Ce qui change concrètement :
- Sideload Moonlight sur Fire OS (pas systématiquement sur l'Amazon Appstore selon la région → passer par l'app *Downloader*)
- La télécommande Fire TV n'a que D-pad / Select / Retour — **pas de boutons façon manette**. La navigation du shell doit rester pilotable avec ce minimum ; une vraie manette Bluetooth reste nécessaire pour **jouer**, pas pour **naviguer**

---

## Principe directeur pour la suite

À chaque nouvelle idée qui arrive (et il y en aura d'autres) : **on l'évalue honnêtement avant de l'ajouter au plan.** Faisable et documenté → on fonce (Spotify, Rich Presence). Séduisant mais risqué ou contraire aux CGU d'un service tiers → on le dit clairement et on cherche le détour légitime (Discord vocal → afficher le vrai client plutôt que le réimplémenter). Pas de promesse qu'on ne peut pas tenir proprement.

---

## Suivi détaillé

Le suivi tâche par tâche (état, priorité, domaine) vit dans Notion, pas ici — ce fichier est la version stable et versionnée du plan, pas le board vivant. Voir la référence `telos-liens` dans la mémoire du projet pour l'URL.

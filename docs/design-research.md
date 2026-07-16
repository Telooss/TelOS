# Recherche design — TelOS

> Document fondateur. Le design de TelOS ne sort pas d'un template : il sort d'une analyse des interfaces console qui ont fonctionné, et d'une direction artistique assumée.

---

## Partie 1 — Analyse des interfaces console

Pour chaque système : architecture de l'information, services, langage visuel, caractère UX, et **ce qu'il y a à voler**.

### PlayStation 4 — le rail de récence

**Architecture.** Deux étages. En bas la *content area* : rangée horizontale de tuiles, dernier jeu joué en gros à gauche, le reste défile à droite par récence. En haut (↑) la *function area* : notifications, amis, trophées, réglages, alimentation. ↓ sur un jeu ouvre son contexte.

**Services.** PS Store, bibliothèque, PS Plus, Live from PlayStation, bouton Share câblé au système, party chat, trophées.

**Visuel.** Dégradé bleu nuit, thèmes dynamiques, accent bleu, plat. Typo maison SST, neutre.

**UX.** Rapide, orienté jeu : ce que tu joues est le plus gros objet à l'écran. Défaut réel : le rail devient interminable et les réglages sont enterrés.

**🔪 À voler.** Le pattern « dernier jeu énorme à gauche, le reste défile à droite » — une seule direction à apprendre, parfait à la manette. Et la séparation stricte contenu (bas) / système (haut), jamais mélangés.

### PlayStation 5 — l'éditorial et le contextuel

**Architecture.** Bascule **Jeux / Média** tout en haut : deux mondes séparés. Le jeu sélectionné explose en **hero art plein écran**, avec dessous une étagère de cartes **Activities** qui font entrer directement dans une mission précise (avec progression et durée estimée). Le **Control Center** (bouton PS) est un dock de cartes superposé au jeu, sans le quitter.

**Services.** PS Plus multi-tiers, Store intégré, Activities, aide-au-jeu vidéo, partage d'écran, switch rapide.

**Visuel.** Sombre, mais la couleur vient de la jaquette. Panneaux translucides givrés, beaucoup d'air, très « magazine ». Parallaxe sur le hero art, transitions lentes et lourdes au sens noble.

**UX.** Le jeu est traité comme une œuvre, pas comme une icône. Les Activities cassent la notion de « lancer le jeu » : tu lances une **intention**.

**🔪 À voler.** (1) Le **hero art géant** du jeu sélectionné — c'est ce qui donne le premium instantané. (2) La **bascule de modes en haut**. (3) Le **Control Center en overlay** — directement pertinent ici : agir (couper le stream, voir la latence, changer de jeu) sans casser la session.

### Xbox 360 — la typographie comme interface

**Architecture.** Trois vies : les **Blades** (2005, onglets colorés glissants), le **NXE** (2008, avatars), puis le **dashboard Metro** (2011) — tuiles plates, panorama horizontal, pivots.

**Visuel.** Vert Xbox `#107C10` sur fond sombre, aplats francs. Segoe en **énorme**, hiérarchie typographique brutale.

**UX.** Metro pose un principe : *content over chrome*. Pas de bordures ni de boutons dessinés — le texte **est** l'interface. Défaut : fin de vie saturée de pubs.

**🔪 À voler.** Le seul langage console à avoir fait de la **typographie géante l'élément principal** plutôt qu'une décoration. Brutal, utilitaire, confiant — exactement le registre DedSec.

### Xbox One — l'ambition qui écrase

**Architecture.** Tuiles héritées de Metro, héro géante, grille de **pins**, **Snap** (app collée pendant le jeu), **Guide** en overlay (double-tap bouton Xbox).

**UX.** Voulait être un centre multimédia autant qu'une console. Résultat au lancement : lourd, lent, confus. Shell refait en 2015 pour dégraisser.

**🔪 À voler.** Le **Guide en overlay** et les **pins**. La leçon négative vaut autant : **une console qui veut tout faire devient illisible.** TelOS doit refuser des fonctionnalités.

### Xbox Series X — la synthèse propre

**Architecture.** Fond dynamique à l'art du jeu, rangée de tuiles épinglées, **groupes/collections**, nav horizontale en haut. Le Guide devient une barre d'onglets latérale.

**Services.** Game Pass au centre (le catalogue est un citoyen de première classe, pas une boutique à part). **Quick Resume** : plusieurs jeux suspendus, bascule en secondes.

**🔪 À voler.** Le **switcher de récents en overlay** (alt-tab de jeux), et l'idée que **basculer** compte autant que **lancer**.

### Nintendo DS — la double zone

**Architecture.** Deux écrans : le haut porte l'**ambiant** (horloge, infos), le bas est **tactile** et porte l'**interaction**.

**Visuel.** Blanc, argent, bleus doux. Rond, jouet, accueillant. L'anti-hacker absolu.

**UX.** Le split **info en haut / action en bas** n'a jamais été repris par une console de salon, et il est excellent : jamais à choisir entre voir le contexte et agir.

**🔪 À voler.** Exactement ce split, parce que le cas d'usage le réclame : une **zone ambiante permanente** (hôte, latence, bitrate, heure, batterie) + une zone d'action. Un bandeau de statut vivant, pas un menu enterré.

### Wii — la métaphore

**Architecture.** Le **Wii Menu** = grille de **Chaînes** carrées, comme une grille de programmes TV. Pointeur Wiimote (une souris en l'air). En bas : réglages et **tableau d'affichage** (timeline calendaire de l'activité).

**Visuel.** Blanc, gris clair, halo bleu, minimalisme quasi-Apple.

**UX.** La force n'est pas graphique, elle est **métaphorique** : tout est une *chaîne*, donc tout est compris en 4 secondes.

**🔪 À voler.** La **métaphore forte qui structure tout**. Une bonne métaphore fait 80 % du boulot d'UX.

### Switch — la vitesse comme feature

**Architecture.** Une rangée horizontale d'icônes carrées **énormes** par récence, fond noir uni. Au-dessus : profils, news, eShop. En dessous : barre fine d'icônes système. Au-delà d'une douzaine de jeux → « Tous les logiciels ».

**Visuel.** Noir, rouge Nintendo `#E60012` en accent unique, zéro chrome, zéro dégradé, zéro effet.

**UX.** **La vitesse.** Réponse dans la frame, aucune animation ne fait attendre. Austère au point d'avoir été critiqué — mais c'est l'interface console la plus **respectueuse du temps de l'utilisateur** jamais produite.

**🔪 À voler.** Les **os**. Structurellement, la Switch est déjà ce que TelOS devrait être : rangée de gros jeux par récence, barre système fine, un bouton pour jouer. On ne peut pas faire mieux comme squelette — on peut juste l'habiller autrement.

---

## Les 5 lois transversales

Non négociables, elles sortent de l'analyse ci-dessus :

1. **Le jeu sélectionné est le plus gros objet à l'écran.** (PS4, PS5, Switch) Toute console qui a fait autrement a échoué.
2. **Système et contenu ne se mélangent jamais.** Deux zones distinctes.
3. **L'overlay bat le menu.** Vital pour un shell de streaming : agir sans casser la session.
4. **La récence bat le rangement.** Le rail par récence résout 90 % des lancements.
5. **Une métaphore forte > mille pixels.**

## La contrainte structurante

L'UI sera **compressée en H.265 et affichée sur un écran de téléphone, parfois à 10 Mbps**. Ça exclut :

- le texte fin,
- les dégradés subtils (banding atroce en streaming),
- les détails de 1 px.

Ça impose : **aplats francs, contraste élevé, typo grasse, gros objets.**

Ce n'est pas une gêne, c'est un **cadre esthétique** — et il tombe pile sur le vocabulaire de Metro… et de DedSec.

---

## Partie 2 — Identité visuelle DedSec

### Le préalable

**Il y a deux DedSec.** Celui de *Watch Dogs 2* (San Francisco) est coloré, streetwear, skate, maker-culture, meme-literate, joyeusement punk — leur identité est diégétiquement l'œuvre de **Sitara**, l'artiste du collectif, ce qui explique le street art assumé. Celui de *Legion* (Londres) est plus gris, plus grave, plus « résistance ».

**Ancrage recommandé : WD2.** Plus distinctif, plus fun, et surtout il évite le piège du hacker-en-hoodie-sur-fond-de-Matrix, cimetière du genre.

### Les 5 piliers réels

1. **Street art physique.** Pochoirs, bombe, affiches collées, stickers, scotch. Tout a l'air **fait à la main et posé sur du réel**. Pilier le plus important, et le plus souvent raté.
2. **Glitch / corruption numérique.** Séparation RGB, pixel sorting, datamosh, scanlines, artefacts VHS, texte corrompu, ASCII. Le numérique qui **casse**.
3. **Collage punk / fanzine.** Découpé-collé, typo façon lettre anonyme, photocopie, demi-teintes, tramé dégueu assumé.
4. **Irrévérence pop.** WD2 est *drôle*. DedSec ne se prend pas au sérieux — c'est ce qui les rend punks plutôt qu'edgelords.
5. **Néon sur béton.** Accents ultra-vifs (cyan, magenta, jaune acide, vert) sur gris béton / asphalte / mur sale. Le contraste matière-brute / couleur-criarde **est** la signature.

### Le système décomposé

| Axe | Principe |
|---|---|
| **Couleur** | Base béton/charbon + accents « bombe » très saturés, en petite quantité. Jamais de dégradé propre : de l'**aplat** et de la **surpulvérisation**. |
| **Typo** | Mélange revendiqué : **pochoir** détourné, **condensé gras** pour le choc, **monospace terminal** pour le versant code. Le mélange *est* le style — un fanzine ne respecte pas une seule fonte. |
| **Matière** | Béton, grain, déchirures de papier, scotch, demi-teintes, scanlines CRT, blocs de glitch. |
| **Mouvement** | Coupes sèches, flicker, décalage RGB, texte qui se tape comme un terminal. Les transitions **corrompent** au lieu de fondre. |
| **Voix** | Minuscules, insolent, anti-système, complice. Pas « Chargement en cours » mais `> décryptage_`. |

### Ce qu'on ne reprend PAS

Les logos et l'app du téléphone in-game sont de la **prop de jeu** : dessinée pour être lue en 2 secondes dans une cinématique. Skeuomorphisme daté, clipart de crâne, aucune tenue en tant que système.

**On prend les principes, pas les assets.** Pas de mascotte, pas de crâne repompé. On construit un langage.

---

## Partie 3 — La traduction : DedSec → TelOS

### La thèse

> ## Squelette de Switch. Peau de DedSec.

DedSec est maximaliste, bruyant, illisible s'il pilote la structure. La Switch est structurellement parfaite et esthétiquement vide. **Le chaos en surface, la rigueur en dessous.** Le glitch ne doit jamais coûter une frame de navigation ni un aller-retour de compréhension.

### La métaphore (leçon Wii) — DÉCISION : uniquement au lancement

**Le PC hôte est un système compromis. telOS est le panneau d'intrusion.**

- Les jeux ne sont pas des « applications » → ce sont des **cibles**.
- Lancer n'est pas « démarrer » → c'est **pénétrer**.
- Le PC hôte est un **nœud**, avec un statut de lien.

**Où ça vit, et où ça ne vit pas.** Tranché : cette métaphore vit **uniquement dans la séquence de lancement**. En permanence, elle est fatigante et vieillit en trois jours. Réservée à l'instant où on appuie sur A, elle devient une récompense — exactement le principe déjà posé pour le glitch : *un événement, pas un fond d'écran.*

**Le reste du temps, l'interface parle normalement** : « Jouer », « Bibliothèque », « Réglages », des jeux — pas de « cibles », pas de « pénétrer », pas de « nœud » dans l'UI au repos.

**Pourquoi ça marche malgré tout, et pas ailleurs :** la plomberie réelle **colle** à la fiction. Il y a un *vrai* hôte distant, une *vraie* latence, un *vrai* débit. Le monospace du bandeau de statut n'est pas décoratif — **il affiche les vraies stats Moonlight**, en permanence, sans jouer la métaphore. C'est de la fiction diégétique honnête, réservée à son moment fort, et c'est ce qui la rend forte plutôt que kitsch.

### Le système concret

**Palette**
```
Béton         #0E0E10   fond
Béton clair   #1A1A1D   surfaces
Texte         #E8E8EA
Accent 1      cyan acide       (sélection, liens actifs)
Accent 2      magenta          (accents secondaires, glitch)
Signal        vert             UNIQUEMENT pour "ONLINE"
```
Aplats uniquement — ce qui règle en prime le banding du streaming.

**Typo**
- Condensé gras **énorme** pour les titres de jeux (leçon Metro : le texte *est* l'interface).
- Monospace pour tout le méta : `HOST: ONLINE`, `LINK: 24ms`, `> lancement_`.

**Structure**
- Rail de récence horizontal *(PS4 / Switch)*
- Jaquette sélectionnée en hero géant *(PS5)*
- Bandeau de statut ambiant permanent *(DS)*
- Barre système fine *(Switch)*
- Overlay Control Center *(PS5 / Xbox)* — couper le stream, basculer de jeu

**Glitch, avec discipline**
- Décalage RGB **uniquement** sur la tuile sélectionnée
- Scanline très légère en overlay global
- **Datamosh au lancement** : l'écran se corrompt, `> ACCÈS ACCORDÉ`, le jeu démarre

Le glitch est un **événement**, pas un fond d'écran.

**Salissures**
Grain de béton, pochoir TelOS bombé en coin, stickers légèrement de travers. C'est ce qui fait « fait main » et pas « template ».

### Le risque à surveiller

**Ce style se plante s'il est propre.** Un glitch bien aligné et symétrique, c'est du Bootstrap déguisé. Il faut du désaxé, du mal découpé, de la surpulvérisation qui dépasse. C'est un travail de finition, pas un filtre CSS.

---

## Le nom : telOS

Graphie fixée, diégétique : **`telOS`** — `tel` en minuscules, `OS` en capitales. Calqué sur **ctOS** (Watch Dogs) : même rapport de casse, sauf que ce réseau-ci est le tien. Cette graphie s'applique partout où le nom est écrit en toutes lettres à l'écran (logo, titres, texte de marque) — pas aux identifiants techniques (repo GitHub `TelOS`, package npm `telos`), qui suivent les conventions de leur écosystème.

## Système typographique

- **Interface** (menus, titres de jeux, méta) → **Oxanium** (Google Fonts, OFL, embarquée dans le repo).
- **Identité et événements** (wordmark, séquence de lancement, `ACCÈS ACCORDÉ`) → **Doctor Glitch** (DaFont, licence usage personnel).

**Contrainte de licence, résolue proprement.** Doctor Glitch est gratuite pour un usage personnel — parfaitement légal pour ce projet (perso, non commercial) — mais sa licence n'autorise pas la redistribution du fichier. Elle est donc utilisée **localement, jamais committée** dans le repo (`.gitignore`), avec sa provenance documentée pour pouvoir la retélécharger en 30 secondes si besoin. Même règle si le repo devient public un jour : aucune violation possible, puisque le fichier n'y a jamais été.

**Contrainte technique découverte à l'usage : Doctor Glitch est caps-only.** Elle n'a pas de glyphes minuscules distincts (courant sur les fontes stencil gratuites) — taper `telOS` dedans ressort `TELOS`. Elle ne peut donc pas porter le wordmark seule sans casser la règle de casse ci-dessus.

**Solution retenue : un wordmark composite, pas un compromis.** `tel` en Oxanium fine et grise, discret — `OS` en Doctor Glitch, énorme et corrompu. Ce n'est pas qu'un contournement technique : ça matérialise le concept, le `tel` humain et discret à côté du `OS` système qui pète l'écran. Ça colle en plus au principe fanzine déjà posé plus haut (« le mélange de fontes est le style »).

**Le Ø n'existe pas non plus dans la fonte** (jeu de caractères limité aux glyphes de base). Plutôt que de dépendre d'un glyphe absent, il est **redessiné à la main en SVG** : un tracé à plusieurs points, volontairement irrégulier — pas une barre parfaitement droite façon règle, dans l'esprit du « désaxé » qu'on s'impose partout ailleurs.

**Candidates écartées :** Rubik Glitch et Rubik Glitch Pop (Google Fonts, libres, glyphes casse-mixte complets — techniquement plus simples) ont été testées puis rejetées à l'usage : rendu jugé trop grossier/organique, pas assez « pochoir net » pour porter l'identité.

**Piège à éviter, déjà identifié en pratique :** une proposition de logo inspirée de références IA génériques (crâne en circuits imprimés, halos lumineux doux sur fond clair) a été écartée — elle contredit à la fois la règle « pas de mascotte, pas de crâne » (déjà posée plus haut) et la règle des aplats (les dégradés doux bandent en streaming H.265). Le style DedSec se juge à l'échec de ce genre de proposition : joli en apparence, mais hors-système.

---

## Décisions arrêtées

- [x] **WD2 (coloré)** retenu comme ancrage, pas Legion (gris/résistance).
- [x] **La métaphore d'intrusion** vit uniquement dans la séquence de lancement — voir plus haut.
- [x] **Système typographique** : Oxanium + Doctor Glitch, wordmark composite, Ø dessiné à la main — voir plus haut.

## Questions ouvertes

- [ ] Logo SVG final : composition posée, **vérification visuelle en attente** (aucun souci de conception — un souci d'outil de capture d'écran côté environnement de dev a interrompu la boucle de vérification en cours de session).

## Références communautaires

Projets réels analysés pour l'inspiration (frontends manette plein écran) :

- [PlayniteModernUI](https://github.com/davidkgriggs/PlayniteModernUI) — moderne, sobre
- [Aniki ReMake](https://github.com/Mike-Aniki/Aniki-ReMake) — console-style très complet
- [gameOS pour Pegasus](https://github.com/PlayingKarrde/gameOS) — la référence épurée
- [Galerie de thèmes Pegasus](https://pegasus-frontend.org/tools/themes/)

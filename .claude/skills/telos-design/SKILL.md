---
name: telos-design
description: Système de design de TelOS — à charger avant d'écrire, modifier ou critiquer la moindre ligne d'UI (HTML/CSS/JS de src/), avant de choisir une couleur, une typo, une animation ou une mise en page, et avant toute maquette ou revue visuelle. Contient les lois structurelles, la direction artistique DedSec/WD2, la liste des réflexes interdits et la boucle de critique obligatoire.
---

# Système de design — TelOS

> Ce document existe parce que le design par défaut est un échec documenté sur ce projet. Il a été rejeté trois fois (« c'est pas beau », « les moodboards sont à chier », « tes standards dégueu et génériques »). Ces règles ne sont pas des suggestions.

---

## Le nom : **telOS**, jamais TelOS ni TELOS

Graphie fixée, diégétique, non négociable : **`telOS`** — préfixe `tel` en **minuscules**, `OS` en **capitales**. Directement calqué sur **ctOS** (Watch Dogs) : dans le jeu, `ct` est le préfixe en minuscules et `OS` reste en capitales. `telOS` reprend exactement ce rapport de casse — c'est un clin d'œil assumé au réseau qui surveille la ville, sauf que celui-ci est le tien.

**Où ça s'applique, sans exception :** logo/wordmark, `<title>`, texte de marque à l'écran, tout endroit où le nom est écrit en toutes lettres dans l'UI.
**Où ça ne s'applique pas :** identifiants techniques (nom de repo GitHub `TelOS`, nom npm `telos`, noms de variables/fichiers) — ces conventions-là suivent les règles de leur écosystème, pas la charte visuelle.

Piège déjà tombé dedans une fois : écrire `TEL` + `OS` (les deux en capitales) dans le markup. Le `tel` doit être **visuellement minuscule à l'écran**, pas juste en CSS `text-transform` qui l'écraserait — écrire le HTML directement en minuscules pour ce mot.

---

## Le logo — VALIDÉ, ne pas redessiner

Wordmark SVG **dessiné à la main**, aucune dépendance à une fonte (source de vérité : `src/logo-test.html`, groupe `#mark`). Validé par Alan. **Deux variantes, et deux seulement :**

| Variante | Où | Traitement |
|---|---|---|
| **Repos** | accueil, UI, favicon | `#mark` tel quel. Propre. |
| **Corrompu** | **intro / chargement uniquement** | `#mark` + datamosh |

**Règles non négociables du logo :**

- **`tel`** : béton gris `#8C8C99`, trait fin, bas de casse. **Reste TOUJOURS intact** — la corruption ne le touche jamais.
- **`OS`** : massif, cyan `#00E5C7`, avec frange chromatique magenta `#FF2D7E` (décalée +5,-4) et jaune `#FFE93D` (décalée -4,+3). **En aplats** — jamais de flou ni de dégradé.
- Le **`O` est un anneau** (`fill:none` + `stroke`), pas une forme pleine. Une pilule pleine ne se lit pas comme une lettre — erreur déjà commise.
- Le **`S` a exactement le même poids de trait que le `O`** (34). Sinon les deux lettres ne font pas bloc.
- Le **`e` est retourné horizontalement** — parti-pris assumé d'Alan. Mais son tracé est **propre et continu** (un seul `path` : barre → panse → terminaison), jamais deux bouts qui se ratent.
- **Le datamosh ne mord que sur le `OS`** (x ≥ 150), avec des **bords irréguliers** (polygone, pas un `rect`). Une bande droite fait « texte biffé », pas corruption.
- **Aucun fragment flottant** qui ne touche aucune lettre : c'est du bruit, pas du glitch. Erreur déjà commise.
- Ne pas trancher les courbes fines (crosses du `S`) avec une découpe : ça les réduit en confettis. Ne couper que les zones pleines.

**Intro au boot** (chantier noté) : le logo se corrompt puis se résout sur la variante repos, façon intro Xbox. Court, sec, skippable — loi zéro.

---

## 0. La boucle obligatoire — ne JAMAIS livrer de l'UI non regardée

La cause racine du mauvais design ici n'est pas le goût, c'est **le fait de coder à l'aveugle**.

```
1. Écrire le code
2. Lancer            → preview_start { name: "telos-dev" }
3. Cadrer            → resize_window { width: 1280, height: 720 }   # TelOS est en 16:9, jamais en portrait
4. REGARDER          → computer { action: "screenshot" }
5. CRITIQUER         → passer la checklist §6, à voix haute, sans complaisance
6. Corriger, revenir à 2
```

**Interdiction absolue** de dire « c'est prêt / voilà l'UI » sans avoir vu un screenshot de l'état final. Un rendu non regardé n'est pas un livrable, c'est une hypothèse.

Vérifier aussi les états, pas seulement l'écran au repos : sélection, survol, séquence de lancement, titre très long, bibliothèque à 1 seul jeu.

---

## 1. Les 5 lois structurelles

Extraites de l'analyse des UI console réelles (`docs/design-research.md`). Non négociables.

1. **Le jeu sélectionné est le plus gros objet à l'écran.** (PS4, PS5, Switch) Toute console ayant fait autrement a échoué.
2. **Système et contenu ne se mélangent jamais.** Deux zones distinctes, toujours.
3. **L'overlay bat le menu.** Agir sans casser la session.
4. **La récence bat le rangement.** Le rail par récence résout 90 % des lancements. Ne pas construire d'arborescence.
5. **Une métaphore forte > mille pixels.**

**La loi zéro, au-dessus des cinq (leçon Switch) : la vitesse est une feature.** La navigation répond dans la frame. Aucune animation ne fait attendre l'utilisateur. Une transition qui retarde une action est un bug, pas du style. Le décor peut être lent ; **la nav, jamais**.

---

## 2. La contrainte qui décide de tout

L'UI est **encodée en H.265 et affichée sur un petit écran, parfois à 10 Mbps sur réseau mobile**.

**Donc c'est interdit, pour des raisons techniques et pas esthétiques :**
- les dégradés subtils → **banding atroce** en streaming
- le texte fin ou petit → bouillie de macroblocs
- les détails de 1 px → mangés par l'encodeur
- les ombres douces et diffuses → bruit de compression
- les grandes zones qui bougent lentement (particules, fonds animés) → mangent le bitrate destiné au jeu

**Donc c'est imposé :** aplats francs, contraste élevé, typo grasse, gros objets, bords nets.

Cette contrainte n'est pas une gêne. **C'est le cadre esthétique**, et il coïncide avec DedSec.

---

## 3. Direction artistique : DedSec, ancrage *Watch Dogs 2*

**Ancrage arrêté : WD2 (San Francisco), pas Legion (Londres).**
Donc : coloré, streetwear, skate, maker-culture, irrévérent, **drôle**. Pas gris, pas grave, pas « résistance ».

### L'erreur à ne surtout pas commettre

**DedSec n'est pas un thème hacker.** Pas de noir profond, pas de vert Matrix, pas de pluie de code, pas de hoodie, pas de cadenas. WD2 est **du béton et de la bombe de peinture colorée**. Si le rendu ressemble à un terminal de film des années 90, c'est raté.

### Les 5 piliers

1. **Street art physique** — pochoirs, bombe, affiches collées, stickers, scotch. Tout a l'air *fait à la main et posé sur du réel*. **Pilier le plus important, le plus souvent raté.**
2. **Glitch / corruption** — décalage RGB, datamosh, scanlines, texte corrompu.
3. **Collage punk / fanzine** — découpé-collé, demi-teintes, photocopie, tramé dégueu assumé.
4. **Irrévérence pop** — DedSec ne se prend pas au sérieux. C'est ce qui les rend punks et pas edgelords.
5. **Néon sur béton** — accents ultra-vifs sur gris béton sale. Le contraste matière-brute / couleur-criarde **est** la signature.

### Tokens

```
/* Béton — la base. Jamais du noir pur : le béton est gris. */
--concrete-900: #16161A;   /* fond */
--concrete-800: #1E1E23;   /* surfaces */
--concrete-700: #2A2A31;   /* bords, séparateurs */
--ink:          #F2F2F4;   /* texte */
--ink-dim:      #8C8C99;   /* texte secondaire, monospace méta */

/* Bombe — très saturé, en PETITE quantité. */
--spray-cyan:    #00E5C7;  /* accent 1 : sélection, focus */
--spray-magenta: #FF2D7E;  /* accent 2 : glitch, secondaire */
--spray-yellow:  #FFE93D;  /* étincelle : à utiliser rarement */
--signal-green:  #6BFF4D;  /* UNIQUEMENT "en ligne" */
--signal-red:    #FF3B30;  /* UNIQUEMENT danger/hors ligne */
```

**Règle de couleur :** aplats uniquement. Jamais de `linear-gradient` décoratif. La couleur vient par grands aplats et par la surpulvérisation qui **dépasse**, pas par des transitions douces.

### Typo

- **Display** : condensé, gras, capitales, interlettrage serré. Le texte **est** l'interface (leçon Metro/Xbox 360), pas une légende.
- **Mono** : tout le méta et les chiffres (`HÔTE EN LIGNE`, `LIEN 24ms`). C'est la texture « code » de DedSec **et** un vrai afficheur.
- **Le mélange de fontes est le style.** Un fanzine ne respecte pas une seule famille. Deux familles qui s'opposent franchement > une famille « harmonieuse ».

### Matière et décor

Grain de béton, demi-teintes, scanline **très** légère, pochoir bombé, stickers **légèrement de travers**, scotch.

> **La règle du désaxé.** Ce style se plante s'il est propre. Un glitch bien aligné et symétrique, c'est du Bootstrap déguisé. Il faut du de travers, du mal découpé, de la surpulvérisation qui dépasse du pochoir. **Si tout est aligné au pixel, c'est raté.**
>
> Mais : le **décor** est désaxé, la **nav** reste rigoureuse. Chaos en surface, rigueur en dessous.

### La thèse du projet

> ## Squelette de Switch. Peau de DedSec.

Structure ascétique, rapide, évidente. Habillage punk. Si le glitch coûte une frame de nav ou un aller-retour de compréhension, il dégage.

---

## 4. Où vit la fiction — décision arrêtée

**La métaphore d'intrusion vit UNIQUEMENT dans la séquence de lancement.** Décision d'Alan, et elle est bonne : en permanence c'est fatigant et ça vieillit en trois jours ; réservée au lancement, elle devient une récompense.

- **L'interface parle normalement** : « Jouer », « Bibliothèque », « Réglages », des jeux. **Pas de « cibles », pas de « pénétrer », pas de « nœud ».**
- **Le lancement est l'événement** : datamosh, décalage RGB, log terminal qui se tape, `ACCÈS ACCORDÉ`, puis le jeu. Court et sec (~1,5 s max — loi zéro).
- Le bandeau de statut en monospace n'est **pas** de la métaphore : il affiche de vraies stats (hôte, latence, débit). C'est ce qui rend l'ensemble honnête plutôt que kitsch.

Même principe partout : **le glitch est un événement, pas un fond d'écran.**

---

## 5. Interdits — mes réflexes par défaut, tous bannis

Cette liste existe parce que je retombe dedans dès que rien ne m'en empêche.

- ❌ **Dégradés bleu→violet** (`#6c8cff` → `#b06cff` et toute la famille). C'est ma signature générique. Bannie.
- ❌ **Glassmorphism** — `backdrop-filter: blur()` sur des panneaux translucides. C'est un template, et ça bande en streaming.
- ❌ **Emojis en guise de jaquettes, d'icônes ou d'illustrations.** Explicitement rejeté. Jamais.
- ❌ **Tout arrondir** (`border-radius: 22px` partout). Le béton et le pochoir ont des **bords nets**.
- ❌ **Ombres douces diffuses** (`box-shadow: 0 10px 30px rgba(0,0,0,.5)`) posées sur tout par défaut.
- ❌ **Fond ambiant flouté qui prend la couleur de l'élément sélectionné.** Déjà fait, déjà rejeté.
- ❌ **Halos lumineux** (`filter: blur(60px)` + `radial-gradient`) pour faire « premium ».
- ❌ **Polices système par défaut** (`system-ui`, Segoe UI) pour le display. C'est un plafond de verre.
- ❌ **Tout centrer** dans une mise en page timide et symétrique.
- ❌ **Cards, cards, cards** — le réflexe grille-de-cartes-arrondies-avec-ombre.
- ❌ **Le mot « moderne »** comme intention de design. Ça ne veut rien dire et ça produit du consensuel.

**Test d'auto-contrôle :** si le rendu pourrait servir de dashboard SaaS en changeant le texte, c'est raté. Il doit être **impossible** de confondre TelOS avec autre chose.

---

## 6. Checklist de critique — à passer sur CHAQUE screenshot

Répondre honnêtement, en nommant ce qui cloche. Une réponse complaisante est un échec.

**Structure**
- [ ] Le jeu sélectionné est-il **manifestement** le plus gros objet à l'écran ?
- [ ] Système et contenu sont-ils dans des zones distinctes ?
- [ ] Est-ce lisible en **1 seconde**, de loin, sur un petit écran ?

**Résistance au streaming**
- [ ] Y a-t-il un seul dégradé qui va bander ? Un texte trop fin ? Un détail à 1 px ?
- [ ] Est-ce que ça tient en contraste si l'image est dégradée ?

**Identité**
- [ ] Est-ce que ça ressemble à **DedSec WD2** — béton + bombe colorée — ou à un thème hacker générique ?
- [ ] Y a-t-il quelque chose de **fait main** à l'écran (pochoir, sticker de travers, scotch, trame) ?
- [ ] Est-ce que quelque chose est **volontairement de travers** ?
- [ ] Est-ce qu'un seul élément de la liste §5 s'est glissé dedans ?
- [ ] Pourrait-on confondre ça avec un dashboard SaaS ? (si oui → refaire)

**Vitesse**
- [ ] Une animation retarde-t-elle une action de l'utilisateur ?

---

## 7. Contexte machine

- Cible : **1280×720 / 1920×1080, paysage**. TelOS s'affiche sur le PC hôte et est streamé. **Jamais de design portrait.**
- L'écran est vu **de loin** (canapé, handheld à bout de bras), pas à 40 cm.
- Dev : `npm run dev` ou `preview_start { name: "telos-dev" }` → http://localhost:5173

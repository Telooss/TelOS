# Fontes

| Fichier | Licence | Committée ? | Rôle |
|---|---|---|---|
| `Oxanium-Variable.ttf` | OFL (libre) | ✅ Oui | Interface — menus, titres de jeux, méta |
| `Oxanium-OFL.txt` | — | ✅ Oui | Texte de licence |
| `DoctorGlitch.otf` | **Usage personnel** (DaFont) | ❌ **Jamais** — gitignorée | Identité — wordmark `OS`, séquence de lancement |
| `RubikGlitch-Regular.ttf`, `RubikGlitchPop-Regular.ttf` | OFL (libre) | ✅ Oui (testées, écartées esthétiquement) | Non utilisées — conservées comme filet de repli si `DoctorGlitch.otf` manque |

## Récupérer Doctor Glitch

Le fichier n'est jamais commité (licence usage personnel : légale pour cet usage, mais pas redistribuable). Après un clone frais du repo, ce fichier manque — c'est normal.

1. Télécharger depuis [DaFont — Doctor Glitch](https://www.dafont.com/doctor-glitch.font) (par Woodcutter).
2. Extraire l'archive, copier le `.otf` ici sous le nom exact `DoctorGlitch.otf`.

Sans ce fichier, `@font-face` échoue silencieusement sur le navigateur/WebView — prévoir un repli visuel (Rubik Glitch, ou Oxanium en gras) plutôt que de bloquer le rendu.

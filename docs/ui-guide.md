# Guide UI

## UI desktop

L'application desktop est conçue pour être autonome. Elle peut charger :

- une fixture JSON ;
- un fichier SQL PostgreSQL ;
- une URL PostgreSQL ;
- la variable d'environnement `DATABASE_URL`.

L'utilisateur sélectionne le type de source, renseigne une seule valeur, charge la structure, puis choisit le schéma et les tables depuis des sélecteurs. Le résultat peut être rendu en `equation`, `sql`, `text`, `mermaid` ou `json`.

## UI web

L'UI web utilise React et appelle l'API Rust. Elle est utile quand on veut une interface navigateur ou une intégration déployée. Contrairement au desktop, elle nécessite un serveur, car le navigateur ne peut pas ouvrir directement une connexion PostgreSQL.

## Principes d'interface

- Un seul champ de source, avec label dynamique selon le type sélectionné.
- Sélecteurs pour les schémas et tables afin d'éviter les fautes de saisie.
- Rendu équation avec saut de ligne à chaque transition `->`.
- Fenêtre desktop librement redimensionnable, sans taille figée.
- Pas de footer ni texte décoratif inutile dans la surface principale.

## Redimensionnement de la fenêtre desktop

La fenêtre Slint ne fixe plus `width` et `height` : dans Slint, poser ces deux propriétés sur un `Window` verrouille la fenêtre à une taille fixe et le gestionnaire de fenêtres refuse tout redimensionnement. Seules les contraintes de layout sont déclarées :

- `preferred-width` / `preferred-height` : taille d'ouverture (1280 x 820).
- `min-width` / `min-height` : taille minimale utilisable (640 x 610), la plus petite fenêtre où la colonne Source, le formulaire Path à hauteur fixe et la colonne de résultat tiennent côte à côte sans être compressés.
- Les panneaux utilisent `min-*`, `preferred-*` et `*-stretch` au lieu de hauteurs et largeurs codées en dur, avec `min-width: 0px` pour pouvoir se réduire.

Le layout s'adapte à toutes les tailles de fenêtre **sans jamais lire la largeur de la fenêtre**
(voir la contrainte Slint ci-dessous) : l'adaptation passe uniquement par les contraintes de layout.

- **Petits écrans** : la fenêtre ne descend pas sous 640 x 610. Le panneau Path aligne un contrôle par ligne, libellé à gauche (`min-width: 88px`), donc aucun sélecteur n'est compressé sous une largeur lisible, même au minimum. La colonne Source se réduit en premier (`min-width: 240px`).
- **Écrans intermédiaires et grands** : la colonne Source garde sa largeur confortable (`preferred-width: 340px`, `max-width: 440px`) et les panneaux Path et Result occupent toute la largeur restante (`horizontal-stretch: 1`). On ne lit pas la taille de l'écran ni `root.width` : cela ferme une boucle de binding Slint.

Le panneau Result absorbe l'espace vertical supplémentaire, et les listes Databases et Tables affichent une barre de défilement uniquement quand leur contenu dépasse.

Les contrôles partagés (sélecteurs de table, sélecteur de format, curseur Max links) sont définis une seule fois dans `main.slint` (`TableSelector`, `FormatSelector`, `MaxLinksBox`) et il n'existe qu'une seule disposition : il n'y a donc plus deux variantes à faire évoluer en parallèle.

### Contrainte Slint à connaître

Slint dimensionne une fenêtre redimensionnable à partir du *layout info* de son contenu. Toute
propriété du layout qui relit la géométrie de la fenêtre referme donc une boucle sur cette taille,
ce que le compilateur signale par :

```text
warning: The binding for the property 'compact' is part of a binding loop
(root.compact -> layoutinfo-h -> root.layoutinfo-h -> root.compact).
         This was allowed in previous version of Slint, but is deprecated and may cause panic at runtime
```

Quatre règles en découlent, verrouillées par `tests/desktop_ui.test.mjs` :

1. **Aucune lecture de la géométrie de la fenêtre** dans `main.slint` : ni `root.width`, ni
   `root.height`, ni `parent.width` / `self.width`, pas même dans la condition d'un `if`. Depuis
   Slint 1.10, les boucles de binding passant par la condition d'un `if` sont détectées : un simple
   booléen de breakpoint suffit à fermer la boucle.
2. **La réactivité passe par les contraintes de layout** : `min-width` / `preferred-width` /
   `max-width`, `horizontal-stretch` / `vertical-stretch`. La seule façon de lire `root.width` sans boucle serait de figer la fenêtre avec
   `width` / `height` — dans Slint, poser ces propriétés donne une taille fixe non redimensionnable —
   ce que le projet refuse.
3. **Pas de ternaires** : les conditions s'écrivent avec des blocs `if`. Un `hauteur: cond ? 48px :
   56px;` enterre une branche au milieu d'une ligne, et c'est par là que les branches dépendant de la
   taille de la fenêtre reviennent. Le test `expresses conditions with blocks instead of ternary
   expressions` vérifie qu'aucun `?` ne subsiste hors commentaire.

4. **Aucun layout de contenu en `alignment: stretch`** (la valeur par défaut). Avec cet
   alignement, Slint taille les enfants `*-stretch: 0` à leur taille **minimale** et, si tous les
   facteurs sont à 0, répartit l'espace restant **à parts égales** : c'est ce qui gonflait les
   sélecteurs verticalement sur les premières captures. Les layouts de contenu déclarent donc
   `alignment: start` (chaque ligne garde sa hauteur préférée), le panneau Path a une hauteur fixe
   (`min-height` = `max-height` = 360px) et seul le panneau Result (`vertical-stretch: 1`) ou un
   espaceur dédié absorbe l'espace supplémentaire.

Ces règles sont revérifiées à chaque build : le job `tests` de
`.github/workflows/build-binaries.yml` recompile l'interface desktop et échoue dès que la sortie
contient « binding loop ».

## Parcours conseillé

1. Choisir le type de source.
2. Charger la structure.
3. Sélectionner le schéma.
4. Choisir la table de départ et la table cible.
5. Choisir le format de sortie.
6. Régler « Max links » : nombre maximal de liens par chemin (5 par défaut, de 1 à 8).
7. Lancer la recherche de chemin.

La recherche affiche **tous** les chemins trouvés, du plus simple au plus complexe :
les chemins à 1 lien d'abord, puis ceux à 2 liens, etc., chacun numéroté
(`Path 1`, `Path 2`, ...). Cela permet de choisir un chemin qui évite une table
non encore renseignée au moment de la saisie. Le curseur « Max links » limite la
longueur maximale des chemins listés.

## Limites actuelles

- Le mode web dépend de l'API Rust.
- Le mode desktop ne lance pas de backend intégré.
- La recherche est basée sur les foreign keys déclarées, pas sur une déduction automatique à partir des noms de colonnes.

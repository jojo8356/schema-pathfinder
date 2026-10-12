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
- `min-width` / `min-height` : taille minimale utilisable (420 x 380), assez petite pour un demi-écran ou une fenêtre courte.
- Les panneaux utilisent `min-*`, `preferred-*` et `*-stretch` au lieu de hauteurs et largeurs codées en dur, avec `min-width: 0px` pour pouvoir se réduire.

Le layout est responsive autour d'un seuil structurel déclaré sur la fenêtre, et d'un
plafond de largeur pour l'espace de travail :

```slint
property <bool> compact: root.width < 900px;
property <length> content-max-width: 1800px;
property <length> header-height: root.compact ? 48px : 56px;
```

- **Au-dessus de 900 px** : la colonne Source reste à gauche et les panneaux Path et Result occupent la largeur restante. Les trois sélecteurs Schema / From table / To table sont alignés en colonnes, et Format / Max links / Find path tiennent sur une seule ligne.
- **En dessous de 900 px** (`compact`) : l'interface bascule sur une seule colonne défilante, chaque contrôle du panneau Path passe sur sa propre ligne avec son libellé à gauche, et le bouton Find path est aligné à droite. Aucun sélecteur n'est compressé sous une largeur lisible, même à 420 px.
- **Grands écrans** : l'espace de travail cesse de s'étirer. Il est centré et plafonné à 1800 px (`Math.min(parent.width, root.content-max-width)` avec `x: (parent.width - self.width) / 2`), et la colonne Source garde une largeur confortable (260 à 440 px) au lieu de suivre la fenêtre. Les lignes monospace du résultat restent lisibles au lieu de traverser tout l'écran.

Le panneau Result absorbe l'espace vertical supplémentaire, et les listes Databases et Tables affichent une barre de défilement uniquement quand leur contenu dépasse.

Les contrôles partagés par les deux dispositions (sélecteurs de table, sélecteur de format, curseur Max links) sont définis une seule fois dans `main.slint` (`TableSelector`, `FormatSelector`, `MaxLinksBox`) pour que le layout large et le layout étroit ne puissent pas diverger.

### Contrainte Slint à connaître

Slint dimensionne la fenêtre à partir du *layout info* du contenu. Lier une largeur, un padding ou
un `font-size` à `root.width` referme donc une boucle sur la largeur de la fenêtre, ce que le
compilateur signale par :

```text
warning: The binding for the property 'font-size' is part of a binding loop
(root.wide -> font-size -> layoutinfo-h -> ... -> root.layoutinfo-h -> root.wide).
         This was allowed in previous version of Slint, but is deprecated and may cause panic at runtime
```

`root.width` ne peut donc piloter que des choix **structurels** (les branches `if root.compact`) et
des tailles **verticales** (`header-height`). Les paddings et les largeurs sont des constantes
définies à l'intérieur de chaque branche. Un test Node
(`never derives a horizontal size, padding or font from the window width`) verrouille cette règle.

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

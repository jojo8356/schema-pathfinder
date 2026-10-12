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

Le layout est responsive et suit deux seuils déclarés sur la fenêtre :

```slint
property <bool> compact: root.width < 900px;
property <bool> wide: root.width >= 1500px;
```

- **Au-dessus de 900 px** : la colonne Source reste à gauche et les panneaux Path et Result occupent la largeur restante. Les trois sélecteurs Schema / From table / To table sont alignés en colonnes, et Format / Max links / Find path tiennent sur une seule ligne.
- **En dessous de 900 px** (`compact`) : l'interface bascule sur une seule colonne défilante, et chaque contrôle du panneau Path passe sur sa propre ligne avec son libellé à gauche. Aucun sélecteur n'est compressé sous une largeur lisible, même à 420 px.
- **Au-dessus de 1500 px** (`wide`) : l'espace de travail cesse de s'étirer. Il est centré et plafonné à 1800 px (`content-max-width`), la colonne Source s'élargit (jusqu'à 460 px) et la typographie passe de 13 px à 15 px. Les lignes monospace du résultat restent lisibles au lieu de traverser tout l'écran.
- Les marges suivent le même principe (`gutter` : 12 px en compact, 16 px en normal, 28 px en wide).

Le panneau Result absorbe l'espace vertical supplémentaire, et les listes Databases et Tables affichent une barre de défilement uniquement quand leur contenu dépasse.

Les contrôles partagés par les deux dispositions (sélecteurs de table, sélecteur de format, curseur Max links) sont définis une seule fois dans `main.slint` (`TableSelector`, `FormatSelector`, `MaxLinksBox`) pour que le layout large et le layout étroit ne puissent pas diverger.

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

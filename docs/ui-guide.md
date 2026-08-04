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
- Layout pensé pour un affichage plein écran de bureau.
- Pas de footer ni texte décoratif inutile dans la surface principale.

## Parcours conseillé

1. Choisir le type de source.
2. Charger la structure.
3. Sélectionner le schéma.
4. Choisir la table de départ et la table cible.
5. Choisir le format de sortie.
6. Lancer la recherche de chemin.

## Limites actuelles

- Le mode web dépend de l'API Rust.
- Le mode desktop ne lance pas de backend intégré.
- La recherche est basée sur les foreign keys déclarées, pas sur une déduction automatique à partir des noms de colonnes.

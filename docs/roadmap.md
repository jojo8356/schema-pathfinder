# Roadmap

## Court terme

- Ajouter des fixtures de schémas plus variés.

### Réalisé (v0.1.4)

- **Stabiliser les commandes CLI autour de `tables`, `schemas`, `databases`, `tree` et `path`.**
  Le parsing vit maintenant dans un module pur (`schema_pathfinder::cli`), testable sans lancer le
  binaire : sous-commandes stables avec options avant ou après, `dbs` en alias, aide par commande,
  `--version`, codes de sortie `0`/`1`/`2`, refus des invocations ambiguës (deux sources, deux
  drapeaux historiques, commande inconnue), et les anciens drapeaux `--tables`/`--schemas`/`--tree`
  conservés en compatibilité.
- **Renforcer les tests sur le parser SQL PostgreSQL.**
  Corps dollar-quotés, commentaires, `ALTER TABLE ... DROP CONSTRAINT`, clés composées, schémas
  multiples, noms à trois parties, contraintes sans nom ni colonne référencée : 33 tests
  d'intégration (`tests/sql_metadata.rs`) et 12 tests unitaires blancs dans le module, plus une
  comparaison directe entre `dressshot_schema.sql` et la fixture
  `dressshot_seed_fk_edges.json`.
- **Améliorer l'affichage desktop sur petits écrans et grands écrans.**
  Fenêtre minimale ramenée à 420 x 380, panneau Path empilé sous 900 px, espace de travail centré et
  plafonné à 1800 px, en-tête plus compact sur fenêtre courte — sans `binding loop` Slint : aucune
  taille horizontale, padding ou `font-size` ne dépend de la largeur de la fenêtre.

## Moyen terme

- Export d'un rapport Markdown ou HTML depuis le CLI.
- Pondération plus explicable des chemins candidats.
- Filtrage par schéma dans le CLI et l'API.
- Mode Mermaid enrichi pour visualiser les tables intermédiaires.
- Documentation d'installation par paquet Debian et AppImage avec captures.

## Long terme

- Heuristiques optionnelles pour détecter des relations probables non déclarées.
- Connecteurs pour d'autres bases relationnelles.
- Mode audit pour signaler les tables isolées ou les relations manquantes.
- UI de comparaison entre deux versions de schéma.

## Non-objectifs actuels

- Lire ou profiler les données métier.
- Modifier le schéma de la base.
- Remplacer un outil de modélisation complet.
- Construire une plateforme SaaS multi-tenant avant stabilisation du produit local.

# Roadmap

## Court terme

- Stabiliser les commandes CLI autour de `tables`, `schemas`, `databases`, `tree` et `path`.
- Renforcer les tests sur le parser SQL PostgreSQL.
- Améliorer l'affichage desktop sur petits écrans et grands écrans.
- Ajouter des fixtures de schémas plus variés.

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

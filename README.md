# Schema Pathfinder

Standalone PostgreSQL schema pathfinder.

Schema Pathfinder est un outil indépendant pour comprendre rapidement comment deux tables PostgreSQL peuvent être reliées par des clés étrangères. Le projet vise un usage très concret : donner à un développeur, un analyste ou un administrateur une réponse exploitable à la question suivante :

```text
Comment joindre la table A à la table B ?
```

Le dépôt contient un CLI Rust, une UI desktop Rust/Slint, une UI web React et une API Rust légère pour le mode web. Le coeur fonctionnel travaille sur les métadonnées de schéma uniquement : il lit les tables, schémas et clés étrangères, sans lire les données métier contenues dans les tables.

## Ce que le projet démontre

- Conception produit d'un outil développeur simple, testable et distribuable.
- Implémentation Rust pour le CLI, l'API locale/web et l'UI desktop.
- Parsing de DDL PostgreSQL vers fixture JSON réutilisable.
- Introspection PostgreSQL directe depuis une URL de connexion.
- Recherche de chemin entre tables avec rendu en texte, équations, SQL, Mermaid et JSON.
- Packaging utilisateur avec AppImage et paquet Debian.
- Documentation pensée pour lecture rapide par un employeur.

## Fonctionnalités principales

- Lister les tables visibles depuis une fixture, un fichier SQL ou une base PostgreSQL.
- Lister les schémas d'une base PostgreSQL.
- Lister les bases accessibles depuis une connexion PostgreSQL.
- Afficher un arbre complet de l'architecture visible : bases, schémas, tables et relations FK.
- Trouver le meilleur chemin de clés étrangères entre deux tables.
- Convertir un fichier SQL PostgreSQL complet en fixture JSON.
- Utiliser le projet en local pur, sans backend, via CLI ou desktop.
- Utiliser le projet en mode web avec une API Rust séparée.

## Exemples rapides

```bash
schema-pathfinder --tables --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder --schemas --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder --databases --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder --tree --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder path ClothingItem User --format equation --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder fixture --sql fixtures/postgres/dressshot_schema.sql
```

Exemple de rendu en mode `equation` :

```text
public.ClothingItem.sellerProfileId = public.SellerProfile.id
->
public.SellerProfile.userId = public.User.id
```

## Surfaces

| Surface | Technologie | Usage |
| --- | --- | --- |
| CLI | Rust, lexopt | Usage offline, scripts, audit rapide en terminal. |
| UI desktop | Rust, Slint | Application locale sans serveur ni URL backend. |
| UI web | React, Vite, TanStack, Lucide | Interface navigateur pour exploration visuelle. |
| API web | Rust, tiny_http | Serveur léger pour alimenter l'UI web. |
| Contrats | Fixtures JSON, tests Node | Validation stable des formats et comportements. |

## Installation développeur

Prérequis : Node.js 22+, pnpm 11+, Rust stable récent, Cargo.

```bash
pnpm install
pnpm test
pnpm typecheck
cargo test -p schema-pathfinder
cargo build --release -p schema-pathfinder --bin schema-pathfinder
```

## Packaging

```bash
node scripts/build_deb.mjs
node scripts/build_appimage.mjs
node scripts/build_desktop_deb.mjs
node scripts/build_desktop_appimage.mjs
```

Les artefacts sont produits dans `dist/` :

- `schema-pathfinder_0.1.3_amd64.deb`
- `schema-pathfinder-0.1.3-x86_64.AppImage`
- `schema-pathfinder-desktop_0.1.3_amd64.deb`
- `schema-pathfinder-desktop-0.1.3-x86_64.AppImage`

## Installation depuis une release

Chaque release GitHub publie un jeu complet d'artefacts construits et vérifiés par CI : binaires Linux, archive `tar.gz`, deux paquets Debian, deux AppImage et `SHA256SUMS`.

```bash
# paquets Debian
sudo apt-get install ./schema-pathfinder_0.1.3_amd64.deb
sudo apt-get install ./schema-pathfinder-desktop_0.1.3_amd64.deb

# ou AppImage, sans installation
chmod +x schema-pathfinder-desktop-0.1.3-x86_64.AppImage
./schema-pathfinder-desktop-0.1.3-x86_64.AppImage
```

## Documentation

- [Vue employeur](docs/employer-overview.md)
- [Architecture](docs/architecture.md)
- [Référence CLI](docs/cli-reference.md)
- [Référence API](docs/api-reference.md)
- [Guide UI](docs/ui-guide.md)
- [Déploiement et packaging](docs/deployment.md)
- [Tests et qualité](docs/testing-and-quality.md)
- [Sécurité et confidentialité](docs/security-and-privacy.md)
- [Roadmap](docs/roadmap.md)
- [Contribution](CONTRIBUTING.md)

## Positionnement sécurité

Schema Pathfinder inspecte la structure d'une base, pas les lignes métier. Les exemples de documentation utilisent des URLs factices. Aucun mot de passe, dump local ou donnée privée ne doit être commité.

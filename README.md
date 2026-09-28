<div align="center">

# Schema Pathfinder

**Trouvez instantanément comment relier deux tables PostgreSQL par leurs clés étrangères.**

CLI Rust · UI desktop Slint · UI web React · API Rust · fixtures & tests de contrat

[![CI](https://github.com/jojo8356/schema-pathfinder/actions/workflows/build-binaries.yml/badge.svg)](https://github.com/jojo8356/schema-pathfinder/actions/workflows/build-binaries.yml)
![Rust](https://img.shields.io/badge/Rust-stable-000000?logo=rust&logoColor=white)
![Node](https://img.shields.io/badge/Node-%3E%3D22-339933?logo=nodedotjs&logoColor=white)
![React](https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=black)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-metadata%20only-4169E1?logo=postgresql&logoColor=white)
![Tests](https://img.shields.io/badge/tests-131%20(66%20Node%20%2B%2065%20Rust)-brightgreen)
![Status](https://img.shields.io/badge/version-0.1.3-blue)

</div>

---

## Sommaire

- [Aperçu](#aperçu)
- [Le problème résolu](#le-problème-résolu)
- [Fonctionnalités](#fonctionnalités)
- [Démarrage rapide](#démarrage-rapide)
- [Surfaces disponibles](#surfaces-disponibles)
- [Formats de sortie](#formats-de-sortie)
- [Utilisation du CLI](#utilisation-du-cli)
- [API web](#api-web)
- [Architecture](#architecture)
- [Développement](#développement)
- [Tests et qualité](#tests-et-qualité)
- [Documentation API (rustdoc)](#documentation-api-rustdoc)
- [Packaging et distribution](#packaging-et-distribution)
- [Sécurité et confidentialité](#sécurité-et-confidentialité)
- [Documentation complète](#documentation-complète)
- [Contribution](#contribution)
- [Licence](#licence)

---

## Aperçu

**Schema Pathfinder** est un outil autonome qui répond à une question très concrète du quotidien
d'un développeur, d'un analyste ou d'un DBA :

> _« Comment joindre la table A à la table B ? »_

L'outil lit **uniquement les métadonnées** d'un schéma PostgreSQL — tables, schémas et clés
étrangères déclarées — puis calcule et classe les chemins de jointure possibles entre deux tables.
Il ne lit jamais les données métier contenues dans les lignes.

Une même logique Rust (`schema_pathfinder::pathfinder_core`) alimente **quatre surfaces** :
un CLI, une application desktop, une interface web et une API HTTP légère.

## Le problème résolu

Dans une base réelle, deux tables sont souvent reliées par **plusieurs chemins de clés étrangères**.
Le chemin le mieux « scoré » n'est pas toujours celui qu'on veut : il peut passer par une table
encore **vide au moment de la saisie** (par exemple une table de facturation non renseignée lorsqu'on
rédige un bon de livraison).

Schema Pathfinder répond à ce besoin en listant **tous les chemins**, **classés du plus simple au
plus complexe** (2 liens, puis 3, puis 4, puis 5…), avec une **limite du nombre de liens
paramétrable par l'utilisateur** (`--max-links`, par défaut `5`). L'utilisateur voit ainsi toutes les
alternatives et choisit celle adaptée à son contexte.

## Fonctionnalités

- 🔎 **Recherche de chemins FK** entre deux tables, classés par nombre de liens croissant.
- 🎚️ **Limite de liens paramétrable** (`--max-links`, défaut 5) sur le CLI, le desktop et le web.
- 🌳 **Arbre d'architecture** complet : bases → schémas → tables → relations FK.
- 📋 **Listing** des tables, schémas et bases visibles.
- 🔌 **Trois sources de données** : fixture JSON, fichier DDL SQL, ou connexion PostgreSQL live.
- 🧩 **Cinq formats de sortie** : `text`, `equation`, `sql`, `mermaid`, `json`.
- 🔄 **Conversion DDL → fixture JSON** réutilisable, sans base en cours d'exécution.
- 🖥️ **Application desktop** locale (Slint), sans serveur ni URL backend.
- 🌐 **Interface web** React + **API Rust** légère (`tiny_http`).
- 📦 **Packaging** prêt à distribuer : `.deb` et `.AppImage` (CLI et desktop).

## Démarrage rapide

Prérequis : **Node.js 22+**, **pnpm 11+**, **Rust stable** et **Cargo**.

```bash
# 1. Cloner et installer
git clone https://github.com/jojo8356/schema-pathfinder.git
cd schema-pathfinder
pnpm install

# 2. Compiler le CLI
cargo build --release -p schema-pathfinder --bin schema-pathfinder

# 3. Trouver un chemin entre deux tables (à partir d'une fixture d'exemple)
./target/release/schema-pathfinder path ClothingItem User \
  --format equation \
  --fixture fixtures/postgres/dressshot_seed_fk_edges.json
```

Sortie (format `equation`) :

```text
public.ClothingItem.sellerProfileId = public.SellerProfile.id
->
public.SellerProfile.userId = public.User.id
```

## Surfaces disponibles

| Surface        | Technologie                      | Usage principal                                           |
| -------------- | -------------------------------- | -------------------------------------------------------- |
| **CLI**        | Rust, `lexopt`                   | Terminal, scripts, audit rapide, CI, mode offline.       |
| **UI desktop** | Rust, Slint                      | Application locale native, sans serveur ni URL backend.  |
| **UI web**     | React 19, Vite, Lucide           | Exploration visuelle dans le navigateur.                 |
| **API web**    | Rust, `tiny_http`                | Serveur HTTP léger qui alimente l'UI web.                |
| **Contrats**   | Fixtures JSON + tests Node/Rust  | Validation stable des formats et comportements.          |

## Formats de sortie

Le format se choisit avec `--format <nom>`. Les cinq formats supportés :

| Format     | Description                                                      |
| ---------- | --------------------------------------------------------------- |
| `text`     | Résumé lisible : score, preuve, longueur du chemin.             |
| `equation` | Suite d'égalités `A.col = B.col` reliant les tables.            |
| `sql`      | Requête `SELECT … JOIN …` prête à exécuter (identifiants cités).|
| `mermaid`  | Diagramme `flowchart LR` pour la documentation.                 |
| `json`     | Sortie structurée, idéale pour l'intégration outillée.         |

## Utilisation du CLI

```text
Usage: schema-pathfinder <command> [options]

Commands:
  tables                   Liste les tables (fixture, SQL ou métadonnées PostgreSQL)
  databases, dbs           Liste les bases PostgreSQL
  schemas                  Liste les schémas de la base sélectionnée
  tree                     Affiche bases, schémas, tables et FK visibles
  fixture                  Génère la fixture JSON depuis la source choisie
  path <source> <target>   Liste les chemins FK entre deux tables, du plus simple au plus complexe

Options:
  --tables | --databases,--dbs | --schemas | --tree   raccourcis de commande
  --database-url <url>     lit les métadonnées depuis cette connexion PostgreSQL
  --fixture <path>         lit les FK depuis une fixture JSON
  --sql <path>             lit un DDL PostgreSQL et le convertit en métadonnées
  --max-links <n>          nombre max de liens FK par chemin listé (défaut 5)
  --format <fmt>           text | equation | sql | mermaid | json
  -h, --help               affiche l'aide

Environment:
  DATABASE_URL             chaîne de connexion utilisée si aucune source explicite n'est fournie
```

Exemples :

```bash
# Lister les tables d'une fixture
schema-pathfinder --tables --fixture fixtures/postgres/dressshot_seed_fk_edges.json

# Introspection d'une base PostgreSQL live
schema-pathfinder --tree     --database-url postgres://readonly:secret@localhost:5432/postgres
schema-pathfinder --schemas  --database-url postgres://readonly:secret@localhost:5432/postgres
schema-pathfinder --databases --database-url postgres://readonly:secret@localhost:5432/postgres

# Chemins entre deux tables, limités à 3 liens, en SQL
schema-pathfinder path ClothingItem User --max-links 3 --format sql \
  --fixture fixtures/postgres/dressshot_seed_fk_edges.json

# Convertir un DDL SQL en fixture JSON
schema-pathfinder fixture --sql fixtures/postgres/dressshot_schema.sql
```

## API web

L'API Rust sert l'UI web compilée et expose des endpoints JSON :

| Méthode | Endpoint         | Description                                       |
| ------- | ---------------- | ------------------------------------------------ |
| `GET`   | `/api/health`    | Sonde de disponibilité.                          |
| `GET`   | `/api/tables`    | Liste des tables de la source configurée.        |
| `GET`   | `/api/databases` | Liste des bases PostgreSQL.                       |
| `GET`   | `/api/schemas`   | Liste des schémas de la base sélectionnée.       |
| `POST`  | `/api/path`      | Chemins FK entre deux tables (source, cible…).   |

```bash
# Compiler et lancer l'ensemble web (UI + API)
pnpm build:saas
cargo run --release -p schema-pathfinder --features api --bin schema-pathfinder-api
```

## Architecture

```
schema-pathfinder/
├── apps/
│   ├── cli-rust/            # Cœur Rust : moteur, CLI et API
│   │   ├── src/
│   │   │   ├── pathfinder_core.rs        # moteur partagé (graphe, scoring, rendus)
│   │   │   ├── main.rs                   # binaire CLI (lexopt)
│   │   │   ├── postgres_sql_metadata.rs  # parsing DDL PostgreSQL
│   │   │   └── bin/schema-pathfinder-api.rs  # API HTTP (tiny_http)
│   │   └── tests/           # tests d'intégration Rust (API publique)
│   ├── desktop/             # UI desktop Rust + Slint
│   └── web/                 # UI web React + Vite
├── packages/
│   ├── core/                # miroir JS du moteur (tests & contrats)
│   └── contracts/           # types de contrat partagés
├── fixtures/postgres/       # schémas d'exemple et attendus de test
├── scripts/                 # build .deb / .AppImage, checks
├── tests/                   # tests Node (contrats, comportements)
└── docs/                    # documentation détaillée
```

Le principe directeur : **une seule implémentation de la logique métier en Rust**
(`pathfinder_core`), réutilisée par le CLI, l'API et le desktop. Un miroir JavaScript sous
`packages/core` sert de banc de tests de contrat et vérifie la stabilité des formats.

## Développement

```bash
pnpm install                                   # dépendances de l'espace de travail
pnpm test                                       # tests Node (66)
pnpm lint                                        # scaffold + contrats
pnpm typecheck                                    # vérification de structure
cargo test -p schema-pathfinder                   # tests Rust (65)
cargo build --release -p schema-pathfinder        # build optimisé du CLI
```

## Tests et qualité

Le projet est couvert par **131 tests** : **66 tests Node** (comportements et contrats) et
**65 tests Rust** (unitaires + intégration).

- Tests unitaires Rust : `#[cfg(test)] mod tests` au sein des modules.
- Tests d'intégration Rust : un fichier par domaine sous `apps/cli-rust/tests/`
  (`path_search`, `scoring`, `renderers`, `sql_metadata`, `architecture_tree`, …).
- Tests Node : `tests/*.test.mjs` (exécutés via le runner intégré `node --test`).
- Doctests : les exemples de la documentation Rust sont exécutés par `cargo test`.

Un **tutoriel complet** explique comment écrire tous ces tests, mesurer la couverture
(`cargo llvm-cov`) et outiller le CLI : voir
[docs/testing-complete-tutorial.md](docs/testing-complete-tutorial.md).

## Documentation API (rustdoc)

Chaque élément public du cœur Rust est documenté. On génère le site HTML complet — une page par
module, type et fonction, avec recherche intégrée :

```bash
pnpm doc:rust          # génère le site dans target/doc/
pnpm doc:rust:open     # génère puis ouvre target/doc/schema_pathfinder/index.html
# équivalent direct :
cargo doc --no-deps -p schema-pathfinder --open
```

Point d'entrée : `target/doc/schema_pathfinder/index.html`.

## Packaging et distribution

```bash
node scripts/build_deb.mjs               # .deb du CLI
node scripts/build_appimage.mjs          # .AppImage du CLI
node scripts/build_desktop_deb.mjs       # .deb du desktop
node scripts/build_desktop_appimage.mjs  # .AppImage du desktop
```

Les artefacts sont produits dans `dist/` :

- `schema-pathfinder_0.1.3_amd64.deb`
- `schema-pathfinder-0.1.3-x86_64.AppImage`
- `schema-pathfinder-desktop_0.1.3_amd64.deb`
- `schema-pathfinder-desktop-0.1.3-x86_64.AppImage`

Installation depuis une release (artefacts construits et vérifiés par CI, avec `SHA256SUMS`) :

```bash
# paquets Debian
sudo apt-get install ./schema-pathfinder_0.1.3_amd64.deb
sudo apt-get install ./schema-pathfinder-desktop_0.1.3_amd64.deb

# ou AppImage, sans installation
chmod +x schema-pathfinder-desktop-0.1.3-x86_64.AppImage
./schema-pathfinder-desktop-0.1.3-x86_64.AppImage
```

## Sécurité et confidentialité

Schema Pathfinder inspecte la **structure** d'une base, jamais les **lignes métier**. Les exemples
de la documentation utilisent des URLs factices. Aucun mot de passe, dump local ni donnée privée ne
doit être commité. Un accès **en lecture seule** suffit pour l'introspection.
Voir [docs/security-and-privacy.md](docs/security-and-privacy.md).

## Documentation complète

- [Vue employeur](docs/employer-overview.md)
- [Architecture](docs/architecture.md)
- [Référence CLI](docs/cli-reference.md)
- [Référence API](docs/api-reference.md)
- [Guide UI](docs/ui-guide.md)
- [Déploiement et packaging](docs/deployment.md)
- [Tests et qualité](docs/testing-and-quality.md)
- [Tutoriel complet : écrire tous les tests](docs/testing-complete-tutorial.md)
- [Sécurité et confidentialité](docs/security-and-privacy.md)
- [Roadmap](docs/roadmap.md)
- [Contribution](CONTRIBUTING.md)

## Contribution

Les contributions sont les bienvenues. Merci de lire [CONTRIBUTING.md](CONTRIBUTING.md) et de
vérifier que `pnpm test`, `pnpm lint` et `cargo test -p schema-pathfinder` passent avant toute
pull request.

## Licence

Aucune licence n'est encore déclarée pour ce dépôt. Tant qu'un fichier `LICENSE` n'est pas ajouté,
le code reste « tous droits réservés » par défaut. Contactez le mainteneur pour tout usage.

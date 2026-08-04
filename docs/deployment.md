# Déploiement et Packaging

## Modes de distribution

Schema Pathfinder peut être livré sous plusieurs formes :

- CLI Rust autonome.
- Application desktop AppImage.
- Application desktop Debian.
- API Rust seule.
- Bundle web + API Rust via Docker.

## Build CLI

```bash
cargo build --release -p schema-pathfinder --bin schema-pathfinder
```

Binaire généré :

```text
target/release/schema-pathfinder
```

## Build API

```bash
cargo build --release -p schema-pathfinder --features api --bin schema-pathfinder-api
```

Binaire généré :

```text
target/release/schema-pathfinder-api
```

## Build web

```bash
pnpm build:web
```

## Packaging Linux

```bash
node scripts/build_deb.mjs
node scripts/build_appimage.mjs
node scripts/build_desktop_deb.mjs
node scripts/build_desktop_appimage.mjs
```

Les scripts produisent les artefacts dans `dist/`.

## Docker

Le Dockerfile construit l'UI web puis l'API Rust. Ce mode est destiné à une exposition contrôlée derrière un reverse proxy ou une plateforme de déploiement.

Points à vérifier en production :

- `DATABASE_URL` fourni côté runtime si l'API doit inspecter une base par défaut.
- `SCHEMA_PATHFINDER_API_TOKEN` activé si l'instance est accessible depuis Internet.
- URL publique et CORS adaptés à l'environnement.
- Accès réseau depuis le conteneur vers PostgreSQL.

## Règle importante

Le CLI et le desktop ne doivent pas dépendre d'une URL backend. Ils sont distribués comme outils locaux indépendants.

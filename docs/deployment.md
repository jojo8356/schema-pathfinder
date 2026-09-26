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

Les paquets Debian déclarent leurs dépendances runtime :

- `schema-pathfinder` : `libc6`, `libgcc-s1`.
- `schema-pathfinder-desktop` : bibliothèques Fontconfig, X11/XCB, xkbcommon, Wayland et GL nécessaires au runtime Slint.

## Release GitHub

Le workflow `.github/workflows/build-binaries.yml` construit les binaires, génère les deux paquets `.deb`, vérifie leur installation (`apt-get install ./dist/*.deb`) puis publie les artefacts.

Deux façons de publier :

```bash
# 1. publication déclenchée par un tag
git tag v0.1.2
git push origin v0.1.2

# 2. publication manuelle vers une release existante
gh workflow run build-binaries.yml --ref <branche> --field release_tag=v0.1.2
```

Assets publiés sur la release :

```text
schema-pathfinder-linux-x86_64
schema-pathfinder-desktop-linux-x86_64
schema-pathfinder-linux-x86_64.tar.gz
schema-pathfinder_<version>_amd64.deb
schema-pathfinder-desktop_<version>_amd64.deb
SHA256SUMS
```

Installation depuis la release :

```bash
sudo apt-get install ./schema-pathfinder_0.1.2_amd64.deb
sudo apt-get install ./schema-pathfinder-desktop_0.1.2_amd64.deb
```

## Docker

Le Dockerfile construit l'UI web puis l'API Rust. Ce mode est destiné à une exposition contrôlée derrière un reverse proxy ou une plateforme de déploiement.

Points à vérifier en production :

- `DATABASE_URL` fourni côté runtime si l'API doit inspecter une base par défaut.
- `SCHEMA_PATHFINDER_API_TOKEN` activé si l'instance est accessible depuis Internet.
- URL publique et CORS adaptés à l'environnement.
- Accès réseau depuis le conteneur vers PostgreSQL.

## Règle importante

Le CLI et le desktop ne doivent pas dépendre d'une URL backend. Ils sont distribués comme outils locaux indépendants.

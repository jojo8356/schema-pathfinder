# Contribution

## Préparer l'environnement

```bash
pnpm install
cargo test -p schema-pathfinder
pnpm test
```

## Style attendu

- Garder le coeur de pathfinding indépendant des surfaces UI.
- Préférer les erreurs avec codes stables.
- Ne pas commiter de secrets, dumps locaux ou URLs PostgreSQL réelles.
- Ajouter ou mettre à jour une fixture quand un comportement de parsing ou de rendu change.
- Documenter les nouvelles commandes CLI dans `docs/cli-reference.md`.

## Avant commit

```bash
pnpm test
pnpm typecheck
cargo test -p schema-pathfinder
```

Pour les changements de packaging :

```bash
node scripts/build_deb.mjs
node scripts/build_appimage.mjs
node scripts/build_desktop_deb.mjs
node scripts/build_desktop_appimage.mjs
```

## Messages de commit

Format conseillé :

```text
feat: add postgres architecture tree command
fix: keep desktop layout scrollable
docs: add employer repository documentation
```

## Revue

Une PR doit expliquer :

- le problème résolu ;
- les surfaces modifiées ;
- les commandes de test lancées ;
- les limites connues si elles existent.

# Sécurité et Confidentialité

## Modèle de sécurité

Schema Pathfinder est conçu pour inspecter des métadonnées de structure : bases visibles, schémas, tables et clés étrangères. Il n'a pas besoin de lire les lignes métier des tables pour remplir son objectif principal.

## Bonnes pratiques PostgreSQL

Créer un rôle dédié avec des droits minimaux :

- connexion à la base ;
- lecture des catalogues système nécessaires ;
- pas de droit d'écriture ;
- pas d'accès large aux données métier si l'environnement ne l'exige pas.

Exemple indicatif :

```sql
CREATE ROLE schema_pathfinder_reader LOGIN PASSWORD 'change_me';
GRANT CONNECT ON DATABASE your_database TO schema_pathfinder_reader;
GRANT USAGE ON SCHEMA public TO schema_pathfinder_reader;
```

Adaptez les droits à votre serveur. Ne réutilisez pas cet exemple tel quel en production.

## Secrets

- Ne jamais commiter d'URL PostgreSQL contenant un vrai mot de passe.
- Utiliser `DATABASE_URL` en variable d'environnement pour les tests locaux.
- Utiliser les secrets runtime de la plateforme pour un déploiement web.
- Supprimer les logs contenant une URL complète avant partage public.

## Web et API

Une instance web exposée peut révéler des métadonnées de schéma. Si elle est accessible publiquement, activez un token ou placez-la derrière une authentification.

## Données SQL générées

Le format `sql` produit un squelette de requête de lecture. L'outil ne l'exécute pas automatiquement. L'utilisateur reste responsable de l'exécution éventuelle dans son environnement.

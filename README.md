# profiled-config

`profiled-config` ambitionne d'apporter aux applications Rust une gestion de
configuration typée et orientée profils, inspirée de l'expérience proposée par
Spring.

L'objectif est de permettre à une application d'embarquer ses fichiers de
configuration TOML dans l'exécutable, de sélectionner un ou plusieurs profils
au démarrage, puis de convertir la configuration obtenue vers une structure
définie par l'utilisateur.

> [!IMPORTANT]
> Le projet en est au stade de la conception. Il n'est pas encore utilisable et
> aucune API n'est stabilisée. Les exemples et comportements décrits ci-dessous
> expriment une direction de travail susceptible d'évoluer.

## Vision

`profiled-config` vise à réunir derrière un point d'entrée unique les capacités
suivantes :

- embarquer dans l'exécutable un dossier complet de fichiers TOML ;
- choisir les profils actifs à l'aide d'arguments de ligne de commande ;
- sélectionner et combiner les fichiers correspondant à ces profils ;
- appliquer les profils dans leur ordre de sélection, les derniers pouvant
  remplacer les clés définies précédemment ;
- résoudre des variables d'environnement depuis la configuration avec une
  syntaxe telle que `${VARIABLE}` ;
- accepter une valeur par défaut, par exemple `${VARIABLE:-default}` ;
- permettre d'écrire un caractère `$` littéral avec `$$` ;
- désérialiser le résultat vers une structure Rust fournie par l'application ;
- exposer l'ensemble du mécanisme au travers d'une macro appliquée au point
  d'entrée.

Le résultat recherché est une configuration reproductible, distribuée avec le
binaire et immédiatement exploitable sous une forme fortement typée.

## Fonctionnement envisagé

À terme, la bibliothèque devrait suivre un flux similaire à celui-ci :

1. les fichiers TOML sont intégrés au binaire pendant la compilation ;
2. l'application détermine les profils actifs lors de son démarrage ;
3. les configurations associées sont fusionnées selon l'ordre des profils ;
4. les expressions faisant référence à l'environnement sont résolues ;
5. la configuration finale est convertie vers le type attendu par
   l'application ;
6. le point d'entrée reçoit une valeur prête à être utilisée.

## Aperçu conceptuel

L'expérience développeur recherchée pourrait ressembler à ceci :

```rust
struct AppConfig {
    creds: String,
}

#[profiled_config]
#[actix_web::main]
async fn main(config: AppConfig) -> std::io::Result<()> {
    // Démarrage de l'application avec une configuration déjà résolue et typée.
    todo!()
}
```

Cet extrait illustre uniquement l'intention du projet. Le nom de la macro, sa
syntaxe, son interaction avec les runtimes asynchrones et les conventions de
nommage des fichiers ou des profils restent à concevoir.

## Principes du projet

- **Typage en premier** : la configuration consommée par l'application doit
  prendre la forme d'un type Rust explicite.
- **Comportement déterministe** : l'ordre de résolution et de remplacement des
  valeurs doit être prévisible et documenté.
- **Binaire autonome** : les configurations connues à la compilation doivent
  pouvoir être distribuées avec l'exécutable.
- **Erreurs compréhensibles** : un profil absent, une variable non résolue ou
  une configuration invalide doit produire un diagnostic utile.
- **Intégration légère** : l'ajout de la bibliothèque à une application doit
  demander le moins de code d'infrastructure possible.

## État du projet

Le périmètre fonctionnel, les choix d'architecture et l'API publique sont
encore ouverts à la discussion. Parmi les sujets à préciser figurent notamment
les conventions de profils, les règles exactes de fusion, la résolution des
variables, les diagnostics de compilation et d'exécution, ainsi que la
compatibilité avec les différents runtimes Rust.

Aucune version publiée ne doit pour l'instant être considérée comme prête pour
un usage en production.

## Contribuer

Le projet a vocation à être développé ouvertement. Les retours sur les cas
d'usage, les attentes ergonomiques et les choix de conception sont les
bienvenus, en particulier tant que les fondations ne sont pas figées.

Avant de proposer une implémentation importante, il est préférable d'ouvrir une
discussion décrivant le besoin, le comportement attendu et les compromis
envisagés.

## Licence

Une licence open source sera choisie avant la première publication du projet.

## Inspiration

Le projet s'inspire des configurations par profils de Spring, sans être affilié
à Spring ni chercher à en reproduire toute l'API.

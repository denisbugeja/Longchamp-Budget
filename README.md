# Longchamp Budget

## Introduction

### Qu’est-ce que Longchamp Budget ?

**Longchamp Budget** est un logiciel destiné aux responsables de groupe et aux pôles administratifs et financiers des groupes scouts.

Il permet de **construire rapidement un budget prévisionnel** pour un groupe, en prenant en compte :

* les unités
* les effectifs
* les dépenses
* les quotients familiaux
* la cotisation nationale

L’objectif est de **calculer de manière fiable et rapide les montants d’adhésion par enfant et par tranche de quotient familial**.

---

### Pourquoi Longchamp Budget ?

Construire un budget scout implique de croiser de nombreuses informations :

* le nombre d’enfants par unité
* les dépenses spécifiques à certaines unités
* les dépenses communes au groupe
* la répartition entre unités et groupe
* les quotients familiaux
* les coefficients multiplicateurs
* la cotisation nationale

Traditionnellement, ces calculs sont réalisés dans **Excel**, ce qui peut devenir :

* long à maintenir
* difficile à modifier
* sensible aux erreurs de formules

Longchamp Budget permet de **centraliser ces données et recalculer instantanément le budget**.

---

### Positionnement de l’outil

Longchamp Budget est un outil de **construction de budget prévisionnel**.
Il ne remplace pas les outils de **comptabilité réelle**.
Il ne remplace pas les outils mise à dispo **au niveau national**.

Il est utilisé **en amont de l’année scoute** pour :

* simuler plusieurs scénarios
* ajuster les dépenses
* préparer l’année suivante
* calculer le montant des adhésions

---

# Principe général du logiciel

Le logiciel repose sur quatre éléments principaux :

* **Les unités**
* **Les dépenses**
* **Les quotients familiaux (QF)**
* **Le budget**

Ces éléments sont configurés puis combinés pour produire :

* le budget par unité
* le budget global du groupe
* les montants d’adhésion par tranche QF

---

# Navigation dans le logiciel

Le logiciel se compose de **quatre tableaux de bord principaux** :

* **Unités**
* **Dépenses**
* **QF**
* **Budget**

Chaque écran correspond à une étape logique de la construction du budget.

Ordre recommandé :

1. Définir les unités
2. Créer les dépenses
3. Configurer les quotients familiaux
4. Construire le budget

---

# Gestion des unités

L’écran **Unités** permet de définir la structure du groupe.

Pour chaque unité il est possible de :

* définir un **nom**
* indiquer le **nombre d’enfants**
* indiquer le **nombre de chefs / cheftaines**
* choisir une **couleur représentative**

Exemples d’unités :

* Farfadets
* Louveteaux / Jeannettes
* Scouts / Guides
* Pionniers / Caravelles
* Compagnons

---

### L’unité "Groupe"

Une unité spéciale appelée **Groupe** est présente par défaut.

Elle :

* regroupe l’ensemble des enfants et des chefs/cheftaines du groupe
* ne peut pas être supprimée
* sert à répartir les **dépenses communes**

---

# Gestion des dépenses

L’écran **Dépenses** permet de créer les postes de dépenses.

Chaque dépense contient :

* un **nom**
* un **prix unitaire**
* un **taux de prise en charge par l’unité**
* une **description (optionnelle)**
* une **association à une ou plusieurs unités (obligatoire)**

---

### Répartition des dépenses

Une dépense peut être :

**100 % unité**

exemple :
goûters des Farfadets

**100 % groupe**

exemple :
achat d’une tente

**partagée**

exemple :
60 % unité
40 % groupe

Dans ce cas :

* la part unité est supportée par l’unité concernée
* la part groupe est répartie entre tous les enfants

---

**Conseil:** chaque dépense comporte un champ **Description**, n'hésitez pas à mémoriser des notes et des commentaires dedans.
Vous pouvez y écrire des tags ( exemple: #chef #weekend #territoire ) afin de pouvoir retrouver plus facilement les lignes concernées grace à l'outil de recherche ( le lien "loupe" situé à droite des liens "Unités", "Dépenses", "QF" et "Budget" ).

---

# Gestion des quotients familiaux (QF)

L’écran **QF** permet de configurer :

* les tranches de quotient familial
* les coefficients multiplicateurs
* la cotisation nationale
* les frais de commission en ligne

Chaque tranche contient :

* un nom
* un coefficient multiplicateur
* la cotisation nationale
* un coefficient de commission
* un montant fixe de commission

---

### Répartition des QF par unité

Pour chaque unité, il est nécessaire d’indiquer :

**le nombre d’enfants par tranche QF**

Ces informations permettent de calculer :

**le tarif moyen pondéré de l’unité**

Plus les données sont précises, plus les montants calculés seront justes.

---

# Construction du budget

L’écran **Budget** est l’espace principal de travail.

Les unités sont affichées sous forme de **sections repliables (accordéons)**.

Pour chaque unité :

* les dépenses disponibles sont listées
* les dépenses peuvent être ajoutées au budget

---

### Ajouter une dépense

Pour ajouter une dépense :

1. sélectionner la dépense
2. cliquer sur le bouton **+**
3. ajuster les paramètres si nécessaire

---

### Paramètres d’une dépense

Chaque dépense possède :

* un **prix unitaire**
* un **nombre d’occurrences**
* le **nombre d’enfants**
* le **nombre de chefs**
* un **taux de prise en charge**

---

**Conseil:** chaque ligne de dépense comporte un champ **Commentaires**, n'hésitez pas à mémoriser des notes et des commentaires dedans.
Vous pouvez y écrire des tags ( exemple: #chef #weekend #territoire ) afin de pouvoir retrouver plus facilement les lignes concernées grace à l'outil de recherche ( le lien "loupe" situé à droite des liens "Unités", "Dépenses", "QF" et "Budget" ).

---

### Règle importante 1

Les **chefs / cheftaines consomment mais ne paient pas**.

Les dépenses incluent les chefs dans la consommation,
mais le coût est réparti **uniquement entre les enfants**.

---

### Règle importante 2

Sur l'écran **Budget**, Longchamp Budget utilise et croise, par défaut, les informations fournies dans les différents écrans du logiciel.
... Mais il est tout à fait possible d'éditer les valeurs suggérées pour la ligne concernée.
Vous pouvez définir une politique globale, mais aussi gérer des cas particuliers.

**Exemple:**

Vous avez défini dans **Dépenses** qu'un weekend coute 14 EUR par tête, et votre unité compte 10 jeunes.
Lorsque vous ajouterez cette ligne de dépense dans votre budget elle affichera par défaut
14 * 10 = 140

Si vous souhaitez ajuster les valeurs uniquement pour cette ligne, vous pouvez remplacer les valeurs suggérées par d'autres valeurs dans les zones de texte. **Il s'agit d'une surcharge de valeur**.

* Cela ne modifiera pas les **valeurs par défaut** pour cette dépense, ou cette unité.
* Vous pourrez revenir aux valeurs par défaut en supprimant les valeurs saisies dans les zones de textes de cette ligne de budget.

---

# Calcul des dépenses

Formule utilisée :

```
Total = Prix unitaire
      × Occurrences
      × (Enfants + Chefs)
      × Taux de prise en charge
```

Le total obtenu est ensuite réparti selon les règles définies.

# Export Excel

Le budget peut être exporté au format **Excel (.xlsx)**.

Le fichier contient :

* un onglet par unité
* un onglet pour les quotients familiaux
* un onglet de balance

L’export permet :

* de partager le budget avec des personnes qui n'ont pas Longchamp Budget
* de présenter les résultats
* de conserver une archive

---

# Préparer l’année suivante

Pour préparer un nouveau budget :

1. copier le fichier `.lb`
2. renommer le fichier (ex : budget-2027.lb)
3. ouvrir le nouveau fichier
4. ajuster :

   * les effectifs
   * les dépenses
   * les prix

Cette méthode permet de **gagner du temps en capitalisant sur les années précédentes**.

---

# Bonnes pratiques

Pour un usage optimal :

* créer à l’avance les dépenses récurrentes
* commenter les dépenses spécifiques
* éviter les surcharges de valeurs si possible
* vérifier la cohérence des répartitions QF

---

# Format des fichiers

Les fichiers Longchamp Budget utilisent l’extension :

```
.lb
```

Ils contiennent l’ensemble du budget et peuvent être :

* copiés
* archivés
* partagés

Conseil: Créez un dossier intitulé "budget **%année%**" sur votre espace cloud (Google Drive, Microsoft OneDrive ... ) et stockez y le fichier budget (.lb) et le fichier d'export Excel (.xlsx).
Vous pourrez lire le contenu du fichier Excel directement dans votre navigateur, et vous garderez précieusement une copie du fichier .lb contenant toutes les informations du budget de l'année.

---

# Philosophie du projet

Longchamp Budget a été développé pour :

* simplifier la construction du budget prévisionnel
* réduire la charge mentale des responsables de groupe
* améliorer la lisibilité financière
* faciliter l'estimation des cotisations des familles.

## Licence
Ce logiciel est **diffusé gratuitement** sous licence GNU GPL v3 - voir le fichier [LICENSE](LICENSE) pour plus de détails.

[![Téléchargez la dernière version](https://img.shields.io/github/v/release/denisbugeja/Longchamp-Budget)](https://github.com/denisbugeja/Longchamp-Budget/releases/latest)
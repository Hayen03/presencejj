= PrésenceJJ -- Guide d'utilisation
Ce petit logicielle permet de générer des fichiers pour les listes de présences et les fiches médicales des enfants du camp.

Il permet également de voir les informations des groupes et de générer différentes statistiques de camp de jours.

== Installation
Le programme utilise Typst pour générer les fichiers pdf, il faut donc que le CLI de Typst soit installé sur l'ordinateur. Typst ainsi que ses instructions d'installation sont disponibles ici: https://github.com/typst/typst

Pour exécuter le programme à la main, il faut l'ouvrir dans un terminal, lui donnant l'emplacement du dossier de templates (remis avec le programme) en argument.

Le programme est disponible dans le OneDrive de Jean-Jeune dans le dossier du camp de jour en cours, avec ses templates et un script qui permet de l'exécuter facilement: Il suffit de double-cliquer sur le script pour l'exécuter.

== Fonctionnement
Le logiciel est un programme console, et n'a donc pas d'interface graphique. À chaque niveau du menu, le programme présente les options disponibles. Il faut alors entrer le numéro de l'option désirée et appuyer sur enter.

Les options du premier menu sont les suivantes:
/ Lire à partir de la programmation: Permet de charger les informations de groupes à partir du fichier de programmation télécharger de Qidigo. Utile pour faire les statistiques de camp, car la programmation permet d'avoir la capacité des groupes.
/ Lire à partir des listes de présences: Permet de charger les enfants inscrits aux différents groupes à partir du fichier de listes de présences téléchargé de Qidigo. Utile pour les statistiques de camp ainsi que pour les fiches médicales et liste de présences hebdomadaires.
/ Faire les sous-groupes: Calcul les sous-groupes, selon la capacité des groupes d'âges et le nombre d'enfant inscrits. Tente de rassembler les enfants par intérêts et former des groupes à profil le plus possible.
/ Faire les fiches médicales: Génère les fiches médicales au format pdf, trié par site de camp et saison, dans le dossier indiqué.
/ Faire les listes de présences: Génère les listes de présences d'animateur et de service de garde au format pdf, trié par saison, site et semaine, dans le dossier indiqué.
/ Estimer la quantité de chandail: Permet d'estimer la quantité de chandails à commander selon le nombre d'enfants présentement inscrits. À deux modes: le mode partiel n'utilise que les enfants présentement inscrits, le mode complet extrapole cette information avec le nombre d'enfants total inscrits les années précédentes (ou l'estimation de l'année en cours).
/ Faire les statistiques de camp: Génère les différentes statistiques de camp (pas encore implémenté).
/ Afficher les données: Affiche les données présentement dans le programme (pas implémenté complètement).
/ Quitter: Quitte le programme.

== Utilisation Générale
Les étapes pour générer les listes de présences et les fiches médicales sont les suivantes:
1. *Télécharger les informations de Qidigo*: Dans l'onglet `Activités > Liste de présences`, sélectionner le modèle approprié contenant toutes les informations nécessaire (présentement le modèle "2025 - Complet"), puis télécharger le fichier excel.
2. *Ouvrir PrésenceJJ*: Ouvrir le programme en double-cliquant sur le script `presencejj.bat`
3. *Lire à partir de la liste de présences*: Sélectionnez l'option 2, puis choisissez le fichier téléchargé à l'étape 1 pour charger les informations.
4. *Faire les sous-groupes*: Sélectionnez l'option 3 pour calculer les sous-groupes. Si le programme est incertain du nombre de sous-groupes à faire, il va vous demander combien vous en voulez pour un groupe donné.
5. *Générer les fiches médicales*: Sélectionnez l'option 4, puis choisissez le dossier de sortie. PrésenceJJ n'écrase pas les fiches médicales déjà existantes, et ne génère que celles des nouveaux enfants. Vous pouvez donc trier par date de création pour n'imprimer que les nouvelles fiches.
6. *Générer les listes de présences*: Sélectionnez l'option 5, puis choisissez le dossier de sortie.
7. *Imprimer les fiches et listes*: Imprimer les documents générés de la manière de votre choix.

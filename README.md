# RAF (رف) — Moteur de Préparation de Commandes Anti-Erreur 🏹📦

Transformez tout smartphone en scanner de fulfillment professionnel. RAF (رف) est conçu pour les grossistes et distributeurs qui veulent éliminer les erreurs de picking et sécuriser leurs expéditions.

## 🚀 La Mission RAF
RAF ne se contente pas de compter ; il **vérifie**. 
En reliant les bons de commande à une interface de scan "zéro-erreur", RAF s'assure que chaque carton contient exactement ce que le client a commandé.

## ✨ Fonctionnalités "Enterprise"
- **Checklist de Fulfillment** : Visualisez en temps réel les articles restants à emballer sur smartphone.
- **Alertes Haptiques "Violentes"** : Vibration SOS immédiate si l'ouvrier scanne un mauvais article.
- **Persistance SQLite** : Toutes les données sont sauvegardées localement dans `raf.db`. Rien n'est perdu au redémarrage.
- **Auto-Découverte (raf.local)** : Couplage mobile instantané sans saisir d'adresse IP.
- **Dashboard Premium** : Interface Bento-style avec suivi de progression en temps réel.
- **Export Pro** : Rapports Excel complets incluant les anomalies et les opérateurs.

## 📖 Flux de Travail
1. **Importation** : Glissez un fichier JSON de commande sur le dashboard.
2. **Scan** : Le préparateur scanne le QR code de la commande.
3. **Validation** : Le préparateur scanne les articles. 
   - ✅ **Vert** : Article correct, progression mise à jour.
   - ❌ **Rouge + Vibration** : Alerte critique si l'article n'est pas dans la commande.
4. **Clôture** : Exportation de la preuve de préparation.

## 🛠 Usage Développeur
```bash
# Lancer le projet
cargo run
```

## 📦 Releases
Téléchargez le binaire Windows pour une utilisation immédiate sans installation depuis l'onglet **Releases**.

## ⚖️ License
MIT

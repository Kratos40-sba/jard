# Jard (جرد) — Le Pont Code-barres Sans Matériel

Transformez n'importe quel PC Windows en serveur Wi-Fi local et utilisez vos smartphones comme scanners de codes-barres haute vitesse synchronisés.

## 🚀 Vision
Jard élimine le besoin de matériel de scan coûteux. En reliant la caméra de votre téléphone à votre PC via le Wi-Fi local, il crée une expérience d'inventaire fluide et sans installation.

## ✨ Caractéristiques
- **Zéro Installation** : Un seul fichier `.exe` contient tout ; l'application mobile est servie directement par le PC.
- **Sécurité Intégrée** : Authentification par jeton unique et protection contre les injections XSS.
- **Multi-Utilisateurs** : Plusieurs employés peuvent scanner simultanément vers le même PC.
- **Auto-Découverte Réseau** : Trouve automatiquement votre IP locale et génère un QR code de couplage.
- **Temps Réel** : Visualisez les scans sur votre bureau au fur et à mesure qu'ils arrivent.
- **Export Excel** : Données propres et agrégées prêtes pour votre logiciel de gestion.

## 📖 Comment ça marche ?
1. **Lancement** : Ouvrez `jard.exe` sur votre PC. Votre navigateur s'ouvrira automatiquement sur le tableau de bord.
2. **Connexion** : Scannez le QR code affiché avec n'importe quel smartphone.
3. **Scan** : Commencez à scanner les codes-barres sur la page mobile.
4. **Export** : Une fois terminé, cliquez sur "Exporter vers Excel" sur votre PC.

## 🛠 Installation (Développeur)
```bash
# Cloner le dépôt
git clone https://github.com/Kratos40-sba/jard.git

# Lancer le projet
cargo run
```

## 📦 Générer le binaire
Pour produire votre propre fichier `.exe` :
1. Créez un tag : `git tag v1.0.0`
2. Poussez le tag : `git push origin v1.0.0`
3. Téléchargez le binaire depuis l'onglet **Releases** sur GitHub.

## ⚖️ License
MIT

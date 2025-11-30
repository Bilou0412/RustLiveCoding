# 42 Storage Manager 🚀

Plus jamais de "Quota Exceeded" ou de "Disk Full" sur ta session.
Cet outil déplace automatiquement tes gros dossiers (`.rustup`, `.vscode`, `.cargo`, `Downloads`) vers le `/goinfre` (SSD) et les sauvegarde sur le `/sgoinfre` (Cloud) le soir.

## ✨ Fonctionnalités

* **Zéro Config :** S'installe en une ligne.
* **Auto-Start :** Restaure ton environnement à chaque login.
* **Rapide :** Utilise `rsync` et le multi-threading pour aller vite.
* **Sûr :** Sauvegarde tes données sur le réseau de l'école.

## 📦 Installation

Oouvre un terminal et copie-colle ça :

```bash
git clone [https://github.com/TON_PSEUDO/42-storage-manager.git](https://github.com/TON_PSEUDO/42-storage-manager.git)
cd 42-storage-manager
./install.sh
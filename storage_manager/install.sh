#!/bin/bash

# Couleurs pour le style
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}    42 STORAGE MANAGER - INSTALLER 🚀    ${NC}"
echo -e "${BLUE}=========================================${NC}"

# 1. Vérification de Rust
if ! command -v cargo &> /dev/null; then
    echo "⚠️  Rust n'est pas installé."
    echo "Installation de Rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# 2. Compilation
echo -e "\n${GREEN}[1/4] Compilation du projet...${NC}"
# On compile en release pour la performance
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Erreur de compilation. Assure-toi d'avoir de l'espace pour compiler !"
    exit 1
fi

# 3. Installation du binaire
echo -e "\n${GREEN}[2/4] Installation dans ~/bin...${NC}"
mkdir -p $HOME/bin
cp target/release/storage_manager $HOME/bin/

# Ajout au PATH si nécessaire (zshrc)
if ! grep -q "$HOME/bin" "$HOME/.zshrc"; then
    echo 'export PATH="$HOME/bin:$PATH"' >> $HOME/.zshrc
    echo "✅ PATH ajouté au .zshrc"
fi

# 4. Configuration du Démarrage Automatique (La magie !)
echo -e "\n${GREEN}[3/4] Configuration du lancement automatique...${NC}"
AUTOSTART_DIR="$HOME/.config/autostart"
mkdir -p "$AUTOSTART_DIR"

# Création du fichier .desktop (Standard Linux pour le startup)
cat <<EOF > "$AUTOSTART_DIR/42_storage_manager.desktop"
[Desktop Entry]
Type=Application
Exec=gnome-terminal -- bash -c "$HOME/bin/storage_manager init; echo '✅ Init terminé. Fermeture...'; sleep 3"
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
Name=42 Storage Manager
Comment=Synchro Goinfre/Sgoinfre automatique
EOF

echo "✅ Fichier autostart créé."

# 5. Premier lancement (Init)
echo -e "\n${GREEN}[4/4] Initialisation immédiate...${NC}"
$HOME/bin/storage_manager init

echo -e "\n${BLUE}=========================================${NC}"
echo -e "${BLUE}   ✨ INSTALLATION TERMINÉE ! ✨        ${NC}"
echo -e "${BLUE}=========================================${NC}"
echo "1. Redémarre ton terminal ou tape 'source ~/.zshrc'"
echo "2. Utilise 'storage_manager save' avant de partir le soir."
echo "3. Au prochain reboot, la fenêtre s'ouvrira toute seule."
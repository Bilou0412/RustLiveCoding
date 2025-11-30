use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::Command;

// --- CONFIGURATION ---
const FOLDERS: &[&str] = &[".rustup", ".vscode", ".cargo", "Downloads"];

#[derive(Parser)]
#[command(name = "42 Storage Manager")]
#[command(about = "Gère la synchro entre Goinfre (SSD) et Sgoinfre (Cloud)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialise le poste (Télécharge depuis Sgoinfre -> Crée les liens)
    Init,
    /// Sauvegarde le travail (Upload vers Sgoinfre)
    Save,
}

fn main() {
    let cli = Cli::parse();

    let user = env::var("USER").expect("Impossible de récupérer l'utilisateur");
    let home_dir = env::var("HOME").expect("Impossible de trouver le HOME");
    let local_storage = format!("/goinfre/{}/local_data", user);
    let remote_storage = format!("/sgoinfre/goinfre/Perso/{}/my_data", user);

    match &cli.command {
        Commands::Init => {
            println!("🚀 Mode INIT activé");
            setup_directories(&local_storage, &remote_storage);
            sync_and_link(&home_dir, &local_storage, &remote_storage);
        }
        Commands::Save => {
            println!("💾 Mode SAVE activé");
            save_work(&local_storage, &remote_storage);
        }
    }
}

// --- FONCTIONS MÉTIERS ---

fn setup_directories(local: &str, remote: &str) {
    let _ = fs::create_dir_all(local);
    let _ = fs::create_dir_all(remote);
}

fn sync_and_link(home: &str, local_root: &str, remote_root: &str) {
    for folder in FOLDERS {
        let local_path = Path::new(local_root).join(folder);
        let remote_path = Path::new(remote_root).join(folder);
        let home_path = Path::new(home).join(folder);
        let backup_path = Path::new(home).join(format!("{}_OLD", folder));

        println!("\n📦 Traitement de : {}", folder);

        // --- ÉTAPE 1 : Préparer le Goinfre (Destination) ---
        if remote_path.exists() {
            // Cas A : Cloud présent → Restauration depuis Sgoinfre
            println!("  📥 Restauration depuis sgoinfre...");
            let source_with_slash = format!("{}/", remote_path.to_str().unwrap());
            run_rsync(&source_with_slash, local_path.to_str().unwrap());
        } else if !local_path.exists() {
            // Cas B : Pas de Cloud → Copier depuis le Home si nécessaire
            let is_real_dir = if let Ok(m) = fs::symlink_metadata(&home_path) {
                !m.file_type().is_symlink()
            } else {
                false
            };

            if is_real_dir {
                println!("  🚚 Copie de sécurité : Home -> Goinfre...");

                // IMPORTANT : Utiliser rsync avec slash pour copier le CONTENU
                // (comme pour la restauration depuis sgoinfre)
                let source_with_slash = format!("{}/", home_path.to_str().unwrap());
                let status = Command::new("rsync")
                    .args(&[
                        "-a",
                        "--info=progress2",
                        &source_with_slash,
                        local_path.to_str().unwrap(),
                    ])
                    .status()
                    .expect("Echec rsync");

                if !status.success() {
                    eprintln!("❌ ERREUR CRITIQUE : La copie a échoué. On ne touche à rien.");
                    continue;
                }
            } else {
                // Cas C : Rien nulle part → Créer un dossier vide
                let _ = fs::create_dir_all(&local_path);
            }
        }

        // --- ÉTAPE 2 : Gérer le Home ---
        if let Ok(metadata) = fs::symlink_metadata(&home_path) {
            if metadata.file_type().is_symlink() {
                println!("  ✅ Lien déjà en place.");
                continue;
            }

            // Si ce n'est pas un lien → Renommer pour sauvegarder
            println!(
                "  🛡️  Sauvegarde locale : Renommage vers {}...",
                backup_path.display()
            );

            if backup_path.exists() {
                let _ = fs::remove_dir_all(&backup_path);
            }

            match fs::rename(&home_path, &backup_path) {
                Ok(_) => println!("  ✓ Renommage réussi."),
                Err(e) => {
                    eprintln!("❌ Impossible de renommer : {}. Arrêt.", e);
                    continue;
                }
            }
        }

        // --- ÉTAPE 3 : Création du lien symbolique ---
        println!("  🔗 Création du lien symbolique...");
        if let Err(e) = symlink(&local_path, &home_path) {
            eprintln!("  ❌ Erreur lien : {}", e);
            // Rollback en cas d'échec
            let _ = fs::rename(&backup_path, &home_path);
        } else {
            println!("  ✨ Succès !");
        }
    }
}

fn save_work(local_root: &str, remote_root: &str) {
    println!("\n☁️  UPLOAD vers Sgoinfre...");
    let source = format!("{}/", local_root);

    let status = Command::new("rsync")
        .args(&["-av", "--info=progress2", &source, remote_root])
        .status()
        .expect("Erreur rsync");

    if status.success() {
        println!("✅ Sauvegarde terminée avec succès !");
    } else {
        eprintln!("⚠️  Erreur lors de la sauvegarde.");
    }
}

fn run_rsync(src: &str, dest: &str) {
    let _ = Command::new("rsync")
        .args(&["-a", "--info=progress2", src, dest])
        .status();
}

use clap::{Parser, Subcommand};
use futures::future::join_all;
use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use tokio::process::Command;

// --- CONFIGURATION ---
const FOLDERS: &[&str] = &[".rustup", ".vscode", ".cargo", "Downloads"];

#[derive(Parser)]
#[command(name = "42 Storage Manager")]
#[command(about = "Version V3 : Architecture Archive (Ultra Rapide)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Save,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let user = env::var("USER").expect("Impossible de récupérer l'utilisateur");
    let home_dir = env::var("HOME").expect("Impossible de trouver le HOME");
    
    // Chemins
    let local_storage = format!("/goinfre/{}/local_data", user);
    // On ajoute un sous-dossier 'archives' pour séparer de l'ancienne version
    let remote_storage = format!("/sgoinfre/goinfre/Perso/{}/my_archives", user);

    match &cli.command {
        Commands::Init => {
            println!("🚀 Mode INIT (Archive Extraction - Vitesse Max)");
            setup_directories(&local_storage, &remote_storage);
            run_parallel_task(&home_dir, &local_storage, &remote_storage, TaskType::Init).await;
        }
        Commands::Save => {
            println!("💾 Mode SAVE (Archivage Parallèle)");
            setup_directories(&local_storage, &remote_storage);
            run_parallel_task(&home_dir, &local_storage, &remote_storage, TaskType::Save).await;
        }
    }
}

fn setup_directories(local: &str, remote: &str) {
    let _ = fs::create_dir_all(local);
    let _ = fs::create_dir_all(remote);
}

#[derive(Clone, Copy)]
enum TaskType {
    Init,
    Save,
}

// Orchestrateur parallèle générique (sert pour Init et Save)
async fn run_parallel_task(home: &str, local_root: &str, remote_root: &str, task: TaskType) {
    let mut tasks = vec![];

    for &folder_name in FOLDERS {
        let home = home.to_string();
        let local_root = local_root.to_string();
        let remote_root = remote_root.to_string();
        let folder = folder_name.to_string();

        tasks.push(tokio::spawn(async move {
            match task {
                TaskType::Init => process_init(&home, &local_root, &remote_root, &folder).await,
                TaskType::Save => process_save(&local_root, &remote_root, &folder).await,
            }
        }));
    }

    join_all(tasks).await;
    println!("✨ Opération terminée !");
}

// --- LOGIQUE INIT (Décompression) ---
async fn process_init(home: &str, local_root: &str, remote_root: &str, folder: &str) {
    let local_path = Path::new(local_root).join(folder);
    let remote_archive = Path::new(remote_root).join(format!("{}.tar", folder));
    let home_path = Path::new(home).join(folder);
    let backup_path = Path::new(home).join(format!("{}_OLD", folder));

    println!("⚡ Start Init : {}", folder);

    // 1. Restauration depuis l'archive (Si elle existe)
    if remote_archive.exists() {
        // Si le dossier local n'existe pas, on décompresse
        if !local_path.exists() {
            let _ = fs::create_dir_all(local_root); // On assure que le parent existe
            
            // Commande : tar -xf /remote/folder.tar -C /local/
            // C'est ultra rapide car on lit un flux continu depuis le réseau
            let status = Command::new("tar")
                .arg("-xf")
                .arg(&remote_archive)
                .arg("-C")
                .arg(local_root)
                .status()
                .await;

            if status.is_ok() && status.unwrap().success() {
                // Succès silencieux pour pas polluer les logs
            } else {
                eprintln!("❌ Erreur extraction {}", folder);
            }
        }
    } else {
        // Fallback : Si pas d'archive (premier lancement), on essaie de copier du Home
        // (Logique Legacy pour transition en douceur)
        if !local_path.exists() {
            let is_real_dir = if let Ok(m) = fs::symlink_metadata(&home_path) {
                !m.file_type().is_symlink()
            } else { false };

            if is_real_dir {
                let _ = fs::create_dir_all(&local_path);
                let src = format!("{}/", home_path.to_str().unwrap());
                let _ = Command::new("rsync").args(&["-a", &src, local_path.to_str().unwrap()]).status().await;
            } else {
                let _ = fs::create_dir_all(&local_path);
            }
        }
    }

    // 2. Gestion des liens (Identique à avant)
    if let Ok(m) = fs::symlink_metadata(&home_path) {
        if !m.file_type().is_symlink() {
            if backup_path.exists() { let _ = fs::remove_dir_all(&backup_path); }
            let _ = fs::rename(&home_path, &backup_path);
        }
    }
    if !home_path.exists() && fs::symlink_metadata(&home_path).is_err() {
        let _ = symlink(&local_path, &home_path);
    }
    
    println!("✅ Ready : {}", folder);
}

// --- LOGIQUE SAVE (Compression) ---
async fn process_save(local_root: &str, remote_root: &str, folder: &str) {
    let local_path = Path::new(local_root).join(folder);
    let remote_archive = Path::new(remote_root).join(format!("{}.tar", folder));

    if !local_path.exists() {
        return; // Rien à sauvegarder
    }

    println!("📦 Start Save (Tar) : {}", folder);

    // Commande : tar -cf /remote/folder.tar -C /local/ folder
    // On écrit directement l'archive sur le réseau.
    // Pas de compression (-z) pour aller plus vite (le réseau de l'école encaisse le débit)
    let status = Command::new("tar")
        .arg("-cf")
        .arg(&remote_archive)
        .arg("-C")
        .arg(local_root)
        .arg(folder)
        .status()
        .await;

    if status.is_ok() && status.unwrap().success() {
        println!("✅ Saved : {}", folder);
    } else {
        eprintln!("⚠️ Erreur sauvegarde {}", folder);
    }
}
use clap::{Parser, Subcommand};
use futures::future::join_all;
use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use tokio::process::Command;

// --- CONFIGURATION ---
const FOLDERS: &[&str] = &[".rustup", ".vscode", ".cargo", "Downloads","Documents"];

#[derive(Parser)]
#[command(name = "42 Storage Manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Save {
        #[arg(long, short)]
        bye: bool,
    },
    Fetch {
        name: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let user = env::var("USER").expect("Impossible de récupérer l'utilisateur");
    let home_dir = env::var("HOME").expect("Impossible de trouver le HOME");
    let local_storage = format!("/goinfre/{}/local_data", user);
    let remote_storage = format!("/sgoinfre/goinfre/Perso/{}/my_archives", user);

    match &cli.command {
        Commands::Init => {
            println!("🚀 Mode INIT");
            setup_directories(&local_storage, &remote_storage);
            run_parallel_task(&home_dir, &local_storage, &remote_storage, TaskType::Init, false).await;
        }
        Commands::Save { bye } => {
            println!("💾 Mode SAVE");
            setup_directories(&local_storage, &remote_storage);
            run_parallel_task(&home_dir, &local_storage, &remote_storage, TaskType::Save, *bye).await;
        }
        Commands::Fetch { name } => {
            println!("📦 Mode FETCH : {}", name);
            setup_directories(&local_storage, &remote_storage);
            fetch_specific_folder(name, &local_storage, &remote_storage).await;
        }
    }
}

fn setup_directories(local: &str, remote: &str) {
    let _ = fs::create_dir_all(local);
    let _ = fs::create_dir_all(remote);
}

#[derive(Clone, Copy)]
enum TaskType { Init, Save }

async fn run_parallel_task(home: &str, local_root: &str, remote_root: &str, task: TaskType, should_logout: bool) {
    
    // 1. LOCK SCREEN AVEC FT_LOCK (Si demandé)
    if let TaskType::Save = task {
        if should_logout {
            lock_session();
        }
    }

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

    // 2. WAIT FOR BACKUP
    join_all(tasks).await;
    println!("✨ Opération terminée !");

    // 3. LOGOUT (Si demandé)
    if let TaskType::Save = task {
        if should_logout {
            trigger_logout();
        }
    }
}

// --- FONCTIONS SYSTÈME ---

fn lock_session() {
    println!("🔒 Verrouillage de l'écran (ft_lock)...");
    
    // TENTATIVE 1 : ft_lock (Officiel École)
    let status = std::process::Command::new("ft_lock").status();

    // Si ft_lock n'existe pas (ex: chez toi), on essaie les méthodes standards
    if status.is_err() {
        println!("⚠️ ft_lock non trouvé, utilisation de gnome-screensaver...");
        let _ = std::process::Command::new("gnome-screensaver-command")
            .arg("-l")
            .status();
        let _ = std::process::Command::new("loginctl")
            .arg("lock-session")
            .status();
    }

    // Petite pause pour laisser le temps au système de verrouiller avant de lancer la charge CPU
    std::thread::sleep(std::time::Duration::from_secs(2));
}

fn trigger_logout() {
    println!("👋 Déconnexion en cours...");
    let _ = std::process::Command::new("gnome-session-quit")
        .arg("--logout")
        .arg("--no-prompt")
        .spawn();
}

// --- LOGIQUE INIT ---
async fn process_init(home: &str, local_root: &str, remote_root: &str, folder: &str) {
    let local_path = Path::new(local_root).join(folder);
    let remote_archive = Path::new(remote_root).join(format!("{}.tar", folder));
    let home_path = Path::new(home).join(folder);
    let backup_path = Path::new(home).join(format!("{}_OLD", folder));

    println!("⚡ Start Init : {}", folder);

    if remote_archive.exists() {
        if !local_path.exists() {
            let _ = fs::create_dir_all(local_root);
            let _ = Command::new("tar").arg("-xf").arg(&remote_archive).arg("-C").arg(local_root).status().await;
        }
    } else {
        // Fallback copie locale
        if !local_path.exists() {
             let is_real = if let Ok(m) = fs::symlink_metadata(&home_path) { !m.file_type().is_symlink() } else { false };
             if is_real {
                let _ = fs::create_dir_all(&local_path);
                let src = format!("{}/", home_path.to_str().unwrap());
                let _ = Command::new("rsync").args(&["-a", &src, local_path.to_str().unwrap()]).status().await;
             } else {
                let _ = fs::create_dir_all(&local_path);
             }
        }
    }

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

// --- LOGIQUE SAVE ---
async fn process_save(local_root: &str, remote_root: &str, folder: &str) {
    let local_path = Path::new(local_root).join(folder);
    let remote_archive = Path::new(remote_root).join(format!("{}.tar", folder));

    if !local_path.exists() { return; }
    println!("📦 Start Save : {}", folder);

    let status = Command::new("tar")
        .arg("-cf").arg(&remote_archive).arg("-C").arg(local_root).arg(folder)
        .status().await;

    if status.is_ok() && status.unwrap().success() {
        println!("✅ Saved : {}", folder);
    } else {
        eprintln!("⚠️ Erreur sauvegarde {}", folder);
    }
}

// --- LOGIQUE FETCH ---
async fn fetch_specific_folder(folder_name: &str, local_root: &str, remote_root: &str) {
    let local_path = Path::new(local_root).join(folder_name);
    let remote_path = Path::new(remote_root).join(folder_name);
    
    if !remote_path.exists() {
        eprintln!("❌ Erreur : '{}' introuvable sur Sgoinfre.", folder_name);
        return;
    }

    println!("📥 Téléchargement de '{}'...", folder_name);
    let _ = fs::create_dir_all(&local_path);
    
    // Pour Fetch, on utilise rsync classique car on suppose que ce ne sont pas des archives
    // (Ou alors tu peux adapter pour gérer les tars aussi ici)
    let src = format!("{}/", remote_path.to_str().unwrap());
    let _ = Command::new("rsync")
        .args(&["-a", "--info=progress2", &src, local_path.to_str().unwrap()])
        .status()
        .await;

    println!("✅ '{}' récupéré !", folder_name);
    let home = env::var("HOME").unwrap();
    let home_link = Path::new(&home).join(folder_name);
    if !home_link.exists() {
        let _ = symlink(&local_path, &home_link);
        println!("🔗 Lien créé.");
    }
}
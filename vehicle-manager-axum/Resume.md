# 📄 FICHE PROJET : MON PREMIER SERVEUR 100% JSON

**Projet :** Vehicle API v1
**Concept clé :** On ne parle qu'en JSON. Le serveur reçoit du JSON et répond en JSON.

-----

## 🛠️ ÉTAPE 1 : PRÉPARATION DU CHANTIER

*Ouvre ton terminal. On part de zéro.*

### 1\. Création

```bash
cargo new my_json_api
cd my_json_api  # ⚠️ N'oublie pas le CD !
```

### 2\. Installation des outils

On a besoin des 3 mousquetaires de Rust :

```bash
# Le serveur web + le moteur asynchrone
cargo add axum tokio --features full

# Le traducteur JSON (Option 'derive' OBLIGATOIRE)
cargo add serde --features derive
```

-----

## 🏗️ ÉTAPE 2 : LE CODE (Main.rs)

*Voici le code complet. Copie-le et regarde les commentaires pour comprendre.*

### A. Les Structures de Données (Les Contrats)

C'est ici qu'on définit la forme de nos données.

  * **`Deserialize` (Entrée)** : Pour lire ce que l'utilisateur envoie.
  * **`Serialize` (Sortie)** : Pour répondre proprement au navigateur/client.

<!-- end list -->

```rust
use axum::{Json, Router, routing::{get, post}};
use serde::{Deserialize, Serialize};

// ENTREE : Ce qu'on attend de l'utilisateur
#[derive(Deserialize)]
struct VehicleData {
    brand: String,
    years: u32,     // Attention au 's', c'est précis !
    model: String,
}

// SORTIE : Notre format de réponse standard
#[derive(Serialize)]
struct ResponseMessage {
    status: String,
    message: String,
}
```

### B. Le Main (Le Chef d'Orchestre)

Il configure les routes et lance le serveur.

```rust
#[tokio::main]
async fn main() {
    // L'aiguillage des routes
    let app = Router::new()
        .route("/ok", get(hello_world))          // Route GET
        .route("/vehicle_data", post(vehicle_data)); // Route POST

    // Démarrage sur le port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Serveur lancé sur http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
```

### C. Les Handlers (La Logique)

Remarque bien : les fonctions renvoient maintenant `Json<ResponseMessage>` et plus du texte brut \!

```rust
// Route GET : Renvoie un JSON de succès
async fn hello_world() -> Json<ResponseMessage> {
    Json(ResponseMessage {
        status: "success".to_string(),
        message: "hello world".to_string()
    })
}

// Route POST : Reçoit une voiture, renvoie une confirmation
async fn vehicle_data(Json(payload): Json<VehicleData>) -> Json<ResponseMessage> {
    // La logique métier (ici on formate juste un message)
    let msg = format!(
        "On a bien save le vehicule: {}, {}, {}",
        payload.brand, payload.model, payload.years
    );

    // La réponse formatée en JSON
    Json(ResponseMessage {
        status: "success".to_string(),
        message: msg,
    })
}
```

-----

## 🧪 ÉTAPE 3 : LE CRASH TEST (Vérification)

*C'est le moment de vérité. Le serveur doit tourner (F5 ou `cargo run`).*

### Test 1 : La route GET (Lecture)

  * **Dans le navigateur :** `http://localhost:3000/ok`
  * **Résultat attendu :**
    ```json
    {"status":"success","message":"hello world"}
    ```

### Test 2 : La route POST (Envoi de données)

  * **Dans le terminal (cURL) :**
    *Copie-colle cette commande exactement (attention aux guillemets sous Windows/Linux).*

<!-- end list -->

```bash
curl -X POST http://localhost:3000/vehicle_data \
   -H "Content-Type: application/json" \
   -d '{"brand": "Peugeot", "model": "208", "years": 2024}'
```

  * **Résultat attendu :**
    ```json
    {
      "status": "success",
      "message": "On a bien save le vehicule: Peugeot, 208, 2024"
    }
    ```

-----

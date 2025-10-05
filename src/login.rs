use reqwest::blocking::Client;
use serde::Deserialize;

use crate::bacheca_personale::get_backeca;

// Struct per deserializzare la risposta JSON del login
#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub api: Api,
    pub data: Data,
    pub error: Vec<String>, // Array di errori (probabilmente stringhe)
    pub time: String,
}

#[derive(Debug, Deserialize)]
pub struct Api {
    #[serde(rename = "AuthSpa")]
    pub auth_spa: AuthSpa,
    pub env: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthSpa {
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub auth: Auth,
    pub pfolio: bool,
}

#[derive(Debug, Deserialize)]
pub struct Auth {
    #[serde(rename = "aMode")]
    pub a_mode: String,
    #[serde(rename = "accountInfo")]
    pub account_info: AccountInfo,
    #[serde(rename = "actionRequested")]
    pub action_requested: bool,
    #[serde(rename = "errCod")]
    pub err_cod: Vec<String>, // Array di codici errore
    pub errors: Vec<String>, // Array di errori
    pub hints: Vec<String>,  // Array di suggerimenti
    #[serde(rename = "loggedIn")]
    pub logged_in: bool,
    #[serde(rename = "mMode")]
    pub m_mode: String,
    pub redirects: Vec<String>, // Array di redirect
    pub verified: bool,
}

#[derive(Debug, Deserialize)]
pub struct AccountInfo {
    pub cid: String,
    pub cognome: String,
    pub id: i32,
    pub nome: String,
    #[serde(rename = "type")]
    pub account_type: String, // "type" è una parola riservata in Rust, rinominata
}

// Funzione per testare se il token di sessione funziona
pub fn test_session_token(
    client: &Client,
    session_id: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    println!("🧪 Testando il token PHPSESSID: {}", session_id);
    match get_backeca(client, session_id) {
        Ok(bacheca) => {
            let circolari_nuove = if let Some(ref msg_new) = bacheca.msg_new {
                msg_new.len()
            } else {
                0
            };
            println!(
                "✅ Token valido - Bacheca caricata con {} circolari lette e {} nuove",
                bacheca.read.len(),
                circolari_nuove
            );
            Ok(true)
        }
        Err(e) => {
            println!("❌ Token scaduto o non valido: {}", e);
            Ok(false)
        }
    }
}

// Funzione per effettuare il login e restituire la session_id
pub fn login(
    client: &Client,
    username: &str,
    password: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let login_action_url = "https://web.spaggiari.eu/auth-p7/app/default/AuthApi4.php?a=aLoginPwd";

    // 1) Prepara i dati del form
    println!("🔐 Preparazione dati login per utente: {}", username);
    let form_data = vec![("uid", username), ("pwd", password)];

    // 2) Invia il form
    println!("📤 Invio credenziali a {}...", login_action_url);
    let res = client.post(login_action_url).form(&form_data).send()?;

    let final_url = res.url().clone();
    let status = res.status();
    let headers = res.headers().clone();
    let response_text = res.text()?;

    // 3) Analizza la risposta del login
    println!("📥 Risposta ricevuta da: {}", final_url);
    println!("📊 Status: {}", status);

    // 3.1) Estrai il PHPSESSID dai cookie della risposta di login
    let mut phpsessid = None;

    // Cerca PHPSESSID negli header Set-Cookie della risposta di login
    for (name, value) in &headers {
        if name.as_str().to_lowercase() == "set-cookie" {
            let cookie_str = value.to_str().unwrap_or("");
            println!("🍪 Set-Cookie: {}", cookie_str);

            if cookie_str.starts_with("PHPSESSID=") {
                // Estrai il valore del PHPSESSID
                let value_part = &cookie_str["PHPSESSID=".len()..];
                if let Some(end_pos) = value_part.find(';') {
                    phpsessid = Some(value_part[..end_pos].to_string());
                } else {
                    phpsessid = Some(value_part.to_string());
                }
                break;
            }
        }
    }

    // 3.2) Analizza il payload JSON usando la struct
    println!("\n📄 Analisi del payload JSON...");

    match serde_json::from_str::<LoginResponse>(&response_text) {
        Ok(login_resp) => {
            println!("✅ Payload JSON deserializzato:");
            println!("  - Ambiente: {}", login_resp.api.env);
            println!("  - Versione AuthSpa: {}", login_resp.api.auth_spa.version);
            println!("  - Logged in: {}", login_resp.data.auth.logged_in);
            println!(
                "  - Account: {} {} (ID: {}, Tipo: {})",
                login_resp.data.auth.account_info.nome,
                login_resp.data.auth.account_info.cognome,
                login_resp.data.auth.account_info.id,
                login_resp.data.auth.account_info.account_type
            );
            println!("  - Tempo: {}", login_resp.time);

            // Verifica se il login è riuscito
            if !login_resp.data.auth.logged_in {
                return Err("Login fallito: loggedIn è false".into());
            }

            // Controlla errori
            if !login_resp.error.is_empty() {
                println!("⚠️ Errori nella risposta: {:?}", login_resp.error);
                return Err(format!("Errori nella risposta: {:?}", login_resp.error).into());
            }
        }
        Err(e) => {
            println!("❌ Errore nel parsing JSON: {}", e);
            println!("📄 Primi 800 caratteri della risposta:");
            println!("{}", &response_text[..response_text.len().min(800)]);
            // Procedi comunque se abbiamo il PHPSESSID
        }
    }

    // 4) Restituisci il PHPSESSID se trovato
    match phpsessid {
        Some(session_id) => {
            println!("✅ PHPSESSID estratto: {}", session_id);

            // Salva il token in un file per uso futuro
            std::fs::write("phpsessid.token", &session_id)?;
            println!("� Token salvato in phpsessid.token");

            Ok(session_id)
        }
        None => {
            println!("❌ PHPSESSID non trovato nei cookie della risposta di login!");
            println!("🔍 Questo potrebbe significare che:");
            println!("  - Il login non è riuscito");
            println!("  - Il server non imposta PHPSESSID in questa risposta");
            println!("  - I cookie sono impostati in un header diverso");

            // Mostra tutti gli header per debug
            println!("\n🔍 Tutti gli header della risposta:");
            for (name, value) in &headers {
                println!("{}: {}", name, value.to_str().unwrap_or("[non-UTF8]"));
            }

            Err("PHPSESSID non trovato nella risposta di login".into())
        }
    }
}

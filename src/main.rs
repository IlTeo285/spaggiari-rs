mod bacheca_personale;

use reqwest::blocking::Client;
use reqwest::cookie::Jar;
use reqwest::header::COOKIE;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::sync::Arc;

use bacheca_personale::{Circolare, download_file, get_backeca, get_comunicazioni};

use crate::bacheca_personale::download_allegati;

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
fn test_session_token(
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
fn login(
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
            std::fs::write("phpsessid.txt", &session_id)?;
            println!("� Token salvato in phpsessid.txt");

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Credenziali di login
    let username = "G13070983V"; // Sostituisci con il tuo username
    let password = "P4azn-P@dJxdt__ra@n7"; // Sostituisci con la tua password

    let jar = Jar::default();
    let jar = Arc::new(jar);
    let client = Client::builder()
        .cookie_provider(jar.clone())
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .build()?;

    // 1) Controlla se esiste già un token salvato
    println!("🔍 Controllo se esiste un token salvato...");
    if let Ok(existing_token) = std::fs::read_to_string("phpsessid.txt") {
        let existing_token = existing_token.trim();
        println!("📁 Token esistente trovato: {}", existing_token);

        // Prova a usare il token esistente
        println!("🧪 Test del token esistente...");
        match test_session_token(&client, existing_token) {
            Ok(true) => {
                println!("✅ Token esistente ancora valido! Uso quello.");

                // Crea la cartella principale download
                fs::create_dir_all("download")?;

                // Ottieni la bacheca
                let bacheca = get_backeca(&client, existing_token)?;

                // Per ogni comunicazione in read e msg_new, elabora
                process_comunicazioni(&client, existing_token, &bacheca.read)?;
                if let Some(msg_new_vec) = &bacheca.msg_new {
                    process_comunicazioni(&client, existing_token, msg_new_vec)?;
                }

                return Ok(());
            }
            _ => {
                println!("❌ Errore durante il test del token esistente");
            }
        }
    } else {
        println!("📁 Nessun token salvato trovato, procedo con il login...");
        println!("\n🔐 Effettuo il login...");
        match login(&client, &username, &password) {
            Ok(session_id) => {
                println!("✅ Login completato con successo!");
                println!("Riesegui il programma per continuare.");
            }
            Err(e) => {
                println!("❌ Login fallito: {}", e);
                return Err(e);
            }
        }
    }

    Ok(())
}

// Nuova funzione per elaborare le comunicazioni
fn process_comunicazioni(
    client: &Client,
    session_id: &str,
    circolari: &[Circolare],
) -> Result<(), Box<dyn std::error::Error>> {
    for circolare in circolari {
        println!(
            "📄 Elaborando comunicazione: {} (Codice: {})",
            circolare.id, circolare.codice
        );

        // Ottieni la comunicazione
        let comunicazione = get_comunicazioni(client, session_id, &circolare.id)?;

        // Crea sottocartella con codice
        let subfolder = format!("download/{}", circolare.codice);
        fs::create_dir_all(&subfolder)?;

        // Scrivi README.txt con il testo
        let readme_path = format!("{}/README.txt", subfolder);
        let mut readme_file = fs::File::create(&readme_path)?;
        readme_file.write_all(comunicazione.testo.as_bytes())?;
        println!("📝 README creato: {}", readme_path);

        // Scarica gli allegati nella sottocartella
        download_allegati(
            client,
            session_id,
            comunicazione.allegati.as_ref(),
            &subfolder,
        )?;
        println!("📂 Allegati scaricati in: {}", subfolder);
    }
    Ok(())
}

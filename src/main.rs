use spaggiari_rs::{bacheca_personale::Circolare, create_client, test_session_token, SpaggiariError, SpaggiariSession};
use std::env;
use std::fs;
use std::io::Write;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), SpaggiariError> {
    // Inizializza tracing
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    // Carica credenziali dalle variabili d'ambiente
    let username = env::var("SPAGGIARI_USERNAME").map_err(|_| {
        error!("❌ Variabile d'ambiente SPAGGIARI_USERNAME non impostata");
        SpaggiariError::EnvVarError("SPAGGIARI_USERNAME".to_string())
    })?;
    let password = env::var("SPAGGIARI_PASSWORD").map_err(|_| {
        error!("❌ Variabile d'ambiente SPAGGIARI_PASSWORD non impostata");
        SpaggiariError::EnvVarError("SPAGGIARI_PASSWORD".to_string())
    })?;

    // 1) Controlla se esiste già un token salvato
    info!("🔍 Controllo se esiste un token salvato...");
    if let Ok(existing_token) = std::fs::read_to_string("phpsessid.token") {
        let existing_token = existing_token.trim();
        info!("📁 Token esistente trovato: {}", existing_token);

        // Prova a usare il token esistente
        info!("🧪 Test del token esistente...");
        let client = create_client()?;
        match test_session_token(&client, existing_token, &username).await {
            Ok(true) => {
                info!("✅ Token esistente ancora valido! Uso quello.");

                // Crea una sessione dal token esistente
                let session = SpaggiariSession::from_token(existing_token.to_string()).await?;

                // Crea la cartella principale download
                fs::create_dir_all("download")?;

                // Ottieni la bacheca
                let bacheca = session.get_bacheca().await?;

                // Per ogni comunicazione in read e msg_new, elabora
                process_comunicazioni(&session, &bacheca.read).await?;
                if let Some(msg_new_vec) = &bacheca.msg_new {
                    process_comunicazioni(&session, msg_new_vec).await?;
                }

                return Ok(());
            }
            Ok(false) => {
                error!("❌ Token esistente non più valido, procedo con nuovo login");
            }
            Err(e) => {
                error!("❌ Errore durante il test del token esistente: {}", e);
            }
        }
    } else {
        info!("📁 Nessun token salvato trovato, procedo con il login...");
    }

    info!("🔐 Effettuo il login...");
    match SpaggiariSession::new(&username, &password).await {
        Ok(session) => {
            info!("✅ Login completato con successo!");

            // Salva il token per la prossima volta
            std::fs::write("phpsessid.txt", &session.session_token)?;
            info!("💾 Token salvato in phpsessid.txt");

            info!("Riesegui il programma per continuare.");
        }
        Err(e) => {
            error!("❌ Login fallito: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

// Nuova funzione per elaborare le comunicazioni usando la sessione
async fn process_comunicazioni(session: &SpaggiariSession, circolari: &[Circolare]) -> Result<(), SpaggiariError> {
    for circolare in circolari {
        info!("📄 Elaborando comunicazione: {} (Codice: {})", circolare.id, circolare.codice);

        // Ottieni la comunicazione
        let comunicazione = session.get_comunicazione(&circolare.id).await?;

        // Crea sottocartella con codice
        let subfolder = format!("download/{}", circolare.codice);
        fs::create_dir_all(&subfolder)?;

        // Scrivi README.txt con il testo
        let readme_path = format!("{}/README.txt", subfolder);
        let mut readme_file = fs::File::create(&readme_path)?;
        readme_file.write_all(comunicazione.testo.as_bytes())?;
        info!("📝 README creato: {}", readme_path);

        // Scarica gli allegati nella sottocartella
        session.download_allegati(&comunicazione.allegati, &subfolder).await?;
        info!("📂 Allegati scaricati in: {}", subfolder);
    }
    Ok(())
}

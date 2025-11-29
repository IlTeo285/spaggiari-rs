use clap::{Parser, Subcommand};
use spaggiari_rs::{bacheca_personale::Circolare, create_client, test_session_token, SpaggiariError, SpaggiariSession};
use std::env;
use std::fs;
use std::io::Write;
use tracing::{error, info};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Effettua il login e salva il token
    Login {
        #[arg(short, long)]
        username: Option<String>,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Verifica se il token salvato è valido
    CheckToken,
    /// Scarica le comunicazioni dalla bacheca
    Download,
    /// Elenca i titoli delle circolari presenti in bacheca
    List,
    /// Mostra i dettagli di una specifica circolare
    Details {
        /// Il codice della circolare da visualizzare
        #[arg(short, long)]
        code: String,
    },
    /// Scarica gli allegati di una specifica circolare
    DownloadCircolare {
        /// Il codice della circolare da scaricare
        #[arg(short, long)]
        code: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), SpaggiariError> {
    // Carica variabili d'ambiente dal file .env se presente
    dotenvy::dotenv().ok();

    // Inizializza tracing
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Login { username, password } => {
            let (user, pass) = get_credentials(username, password)?;
            info!("🔐 Effettuo il login per utente: {}", user);
            match SpaggiariSession::new(&user, &pass).await {
                Ok(session) => {
                    info!("✅ Login completato con successo!");
                    // Salva il token
                    std::fs::write("phpsessid.token", &session.session_token)?;
                    info!("💾 Token salvato in phpsessid.token");
                }
                Err(e) => {
                    error!("❌ Login fallito: {}", e);
                    return Err(e);
                }
            }
        }
        Commands::CheckToken => {
            info!("🔍 Controllo validità del token salvato...");
            if let Ok(existing_token) = std::fs::read_to_string("phpsessid.token") {
                let existing_token = existing_token.trim();
                let username = env::var("SPAGGIARI_USERNAME").unwrap_or_else(|_| "G13070983V".to_string()); // Fallback o errore?

                let client = create_client()?;
                match test_session_token(&client, existing_token, &username).await {
                    Ok(true) => info!("✅ Il token è valido!"),
                    Ok(false) => info!("❌ Il token è scaduto o non valido."),
                    Err(e) => error!("❌ Errore durante il controllo: {}", e),
                }
            } else {
                error!("❌ Nessun token trovato in phpsessid.token");
            }
        }
        Commands::Download => {
            // Logica di download simile a prima
            // 1. Recupera token
            if let Ok(existing_token) = std::fs::read_to_string("phpsessid.token") {
                let existing_token = existing_token.trim();
                info!("📁 Token trovato. Avvio sessione...");

                let session = SpaggiariSession::from_token(existing_token.to_string()).await?;

                // Crea la cartella principale download
                fs::create_dir_all("download")?;

                // Ottieni la bacheca
                info!("📥 Scaricamento bacheca...");
                let bacheca = session.get_bacheca().await?;

                // Per ogni comunicazione in read e msg_new, elabora
                process_comunicazioni(&session, &bacheca.read).await?;
                if let Some(msg_new_vec) = &bacheca.msg_new {
                    process_comunicazioni(&session, msg_new_vec).await?;
                }
                info!("✅ Download completato.");
            } else {
                error!("❌ Nessun token trovato. Esegui prima il login.");
            }
        }
        Commands::List => {
            if let Ok(existing_token) = std::fs::read_to_string("phpsessid.token") {
                let existing_token = existing_token.trim();
                info!("📁 Token trovato. Recupero lista circolari...");

                let session = SpaggiariSession::from_token(existing_token.to_string()).await?;
                let bacheca = session.get_bacheca().await?;

                println!("📋 Elenco Circolari:");
                println!("---------------------------------------------------");

                if let Some(msg_new) = &bacheca.msg_new {
                    for circolare in msg_new {
                        println!("🆕 ID: {} - {}", circolare.id, circolare.titolo);
                    }
                }

                for circolare in &bacheca.read {
                    println!("✅ ID: {} - {}", circolare.id, circolare.titolo);
                }
                println!("---------------------------------------------------");
            } else {
                error!("❌ Nessun token trovato. Esegui prima il login.");
            }
        }
        Commands::Details { code } => {
            if let Ok(existing_token) = std::fs::read_to_string("phpsessid.token") {
                let existing_token = existing_token.trim();
                info!("📁 Token trovato. Recupero dettagli circolare {}...", code);

                let session = SpaggiariSession::from_token(existing_token.to_string()).await?;

                match session.get_comunicazione(&code).await {
                    Ok(comunicazione) => {
                        println!("📄 Dettagli Circolare:");
                        println!("---------------------------------------------------");
                        println!("Testo:\n{}", comunicazione.testo);
                        println!("---------------------------------------------------");
                        if !comunicazione.allegati.is_empty() {
                            println!("📎 Allegati:");
                            for allegato in comunicazione.allegati {
                                println!("  - ID Allegato: {}", allegato.allegato_id);
                            }
                        } else {
                            println!("📎 Nessun allegato.");
                        }
                        println!("---------------------------------------------------");
                    }
                    Err(e) => {
                        error!("❌ Errore nel recupero della comunicazione: {}", e);
                    }
                }
            } else {
                error!("❌ Nessun token trovato. Esegui prima il login.");
            }
        }
        Commands::DownloadCircolare { code } => {
            if let Ok(existing_token) = std::fs::read_to_string("phpsessid.token") {
                let existing_token = existing_token.trim();
                info!("📁 Token trovato. Scarico circolare {}...", code);

                let session = SpaggiariSession::from_token(existing_token.to_string()).await?;

                match session.get_comunicazione(&code).await {
                    Ok(comunicazione) => {
                        let subfolder = format!("download/{}", code);
                        fs::create_dir_all(&subfolder)?;

                        let readme_path = format!("{}/README.txt", subfolder);
                        let mut readme_file = fs::File::create(&readme_path)?;
                        readme_file.write_all(comunicazione.testo.as_bytes())?;
                        info!("📝 README creato: {}", readme_path);

                        if !comunicazione.allegati.is_empty() {
                            session.download_allegati(&comunicazione.allegati, &subfolder).await?;
                            info!("📂 Allegati scaricati in: {}", subfolder);
                        } else {
                            info!("ℹ️ Nessun allegato presente.");
                        }
                    }
                    Err(e) => {
                        error!("❌ Errore nel recupero della comunicazione: {}", e);
                    }
                }
            } else {
                error!("❌ Nessun token trovato. Esegui prima il login.");
            }
        }
    }

    Ok(())
}

fn get_credentials(cli_user: Option<String>, cli_pass: Option<String>) -> Result<(String, String), SpaggiariError> {
    let username = cli_user.or_else(|| env::var("SPAGGIARI_USERNAME").ok());
    let password = cli_pass.or_else(|| env::var("SPAGGIARI_PASSWORD").ok());

    match (username, password) {
        (Some(u), Some(p)) => Ok((u, p)),
        _ => Err(SpaggiariError::Generic("Credenziali mancanti. Usa argomenti CLI o variabili d'ambiente SPAGGIARI_USERNAME/PASSWORD".to_string())),
    }
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

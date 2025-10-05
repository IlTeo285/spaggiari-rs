use spaggiari_rs::{
    bacheca_personale::Circolare, create_client, test_session_token, SpaggiariSession,
};
use std::env;
use std::fs;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Carica credenziali dalle variabili d'ambiente
    let username = env::var("SPAGGIARI_USERNAME")
        .map_err(|_| "Variabile d'ambiente SPAGGIARI_USERNAME non impostata")?;
    let password = env::var("SPAGGIARI_PASSWORD")
        .map_err(|_| "Variabile d'ambiente SPAGGIARI_PASSWORD non impostata")?;

    // 1) Controlla se esiste già un token salvato
    println!("🔍 Controllo se esiste un token salvato...");
    if let Ok(existing_token) = std::fs::read_to_string("phpsessid.token") {
        let existing_token = existing_token.trim();
        println!("📁 Token esistente trovato: {}", existing_token);

        // Prova a usare il token esistente
        println!("🧪 Test del token esistente...");
        let client = create_client()?;
        match test_session_token(&client, existing_token, &username) {
            Ok(true) => {
                println!("✅ Token esistente ancora valido! Uso quello.");

                // Crea una sessione dal token esistente
                let session = SpaggiariSession::from_token(existing_token.to_string())?;

                // Crea la cartella principale download
                fs::create_dir_all("download")?;

                // Ottieni la bacheca
                let bacheca = session.get_bacheca()?;

                // Per ogni comunicazione in read e msg_new, elabora
                process_comunicazioni(&session, &bacheca.read)?;
                if let Some(msg_new_vec) = &bacheca.msg_new {
                    process_comunicazioni(&session, msg_new_vec)?;
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
        match SpaggiariSession::new(&username, &password) {
            Ok(session) => {
                println!("✅ Login completato con successo!");

                // Salva il token per la prossima volta
                std::fs::write("phpsessid.txt", &session.session_token)?;

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

// Nuova funzione per elaborare le comunicazioni usando la sessione
fn process_comunicazioni(
    session: &SpaggiariSession,
    circolari: &[Circolare],
) -> Result<(), Box<dyn std::error::Error>> {
    for circolare in circolari {
        println!(
            "📄 Elaborando comunicazione: {} (Codice: {})",
            circolare.id, circolare.codice
        );

        // Ottieni la comunicazione
        let comunicazione = session.get_comunicazione(&circolare.id)?;

        // Crea sottocartella con codice
        let subfolder = format!("download/{}", circolare.codice);
        fs::create_dir_all(&subfolder)?;

        // Scrivi README.txt con il testo
        let readme_path = format!("{}/README.txt", subfolder);
        let mut readme_file = fs::File::create(&readme_path)?;
        readme_file.write_all(comunicazione.testo.as_bytes())?;
        println!("📝 README creato: {}", readme_path);

        // Scarica gli allegati nella sottocartella
        session.download_allegati(&comunicazione.allegati, &subfolder)?;
        println!("📂 Allegati scaricati in: {}", subfolder);
    }
    Ok(())
}

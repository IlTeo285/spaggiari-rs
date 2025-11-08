use spaggiari_rs::{create_client, SpaggiariSession};
use std::{env, fs};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Carica credenziali dalle variabili d'ambiente
    let username = env::var("SPAGGIARI_USERNAME")
        .map_err(|_| "Variabile d'ambiente SPAGGIARI_USERNAME non impostata. Esegui: export SPAGGIARI_USERNAME=tuo_codice_fiscale")?;
    let password = env::var("SPAGGIARI_PASSWORD")
        .map_err(|_| "Variabile d'ambiente SPAGGIARI_PASSWORD non impostata. Esegui: export SPAGGIARI_PASSWORD=tua_password")?;

    // Esempio 1: Creare una sessione con login
    println!("🔐 Effettuo il login...");
    let session = SpaggiariSession::new(&username, &password).await?;

    // Verifica che la sessione sia valida
    if session.is_valid().await? {
        println!("✅ Sessione valida!");

        // Ottieni la bacheca
        let bacheca = session.get_bacheca().await?;
        println!("📄 Trovate {} comunicazioni lette", bacheca.read.len());

        if let Some(msg_new) = &bacheca.msg_new {
            println!("📬 Trovate {} nuove comunicazioni", msg_new.len());
        }

        // Esempio: elabora le prime 3 comunicazioni
        for (i, circolare) in bacheca.read.iter().take(3).enumerate() {
            println!(
                "\n📄 Comunicazione {}: {} (Codice: {})",
                i + 1,
                circolare.titolo,
                circolare.codice
            );

            // Ottieni i dettagli della comunicazione
            let comunicazione = session.get_comunicazione(&circolare.id).await?;
            println!(
                "📝 Testo: {}",
                comunicazione.testo.chars().take(100).collect::<String>()
            );

            // Crea una cartella per questa comunicazione
            let folder = format!("esempi_download/{}", circolare.codice);
            fs::create_dir_all(&folder)?;

            // Scarica gli allegati se ce ne sono
            if !comunicazione.allegati.is_empty() {
                println!("📎 Scarico {} allegati...", comunicazione.allegati.len());
                session
                    .download_allegati(&comunicazione.allegati, &folder)
                    .await?;
                println!("✅ Allegati scaricati in: {}", folder);
            } else {
                println!("📎 Nessun allegato da scaricare");
            }
        }
    } else {
        println!("❌ Sessione non valida!");
    }

    Ok(())
}

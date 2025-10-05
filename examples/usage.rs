use spaggiari_rs::{create_client, SpaggiariSession};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Esempio 1: Creare una sessione con login
    println!("🔐 Effettuo il login...");
    let session = SpaggiariSession::new("TUO_CODICE_FISCALE", "TUA_PASSWORD")?;

    // Esempio 2: Usare un token salvato (se ne hai uno)
    // let session = SpaggiariSession::from_token("token_esistente".to_string())?;

    // Verifica che la sessione sia valida
    if session.is_valid()? {
        println!("✅ Sessione valida!");

        // Ottieni la bacheca
        let bacheca = session.get_bacheca()?;
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
            let comunicazione = session.get_comunicazione(&circolare.id)?;
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
                session.download_allegati(&comunicazione.allegati, &folder)?;
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

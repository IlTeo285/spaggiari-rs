use spaggiari_rs::SpaggiariSession;
use std::env;

// Funzione helper per rilevare il tipo di file dal contenuto
fn detect_file_type(content: &[u8]) -> &'static str {
    if content.starts_with(b"%PDF") {
        "PDF"
    } else if content.starts_with(b"\x89PNG") {
        "PNG"
    } else if content.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "JPEG"
    } else if content.starts_with(b"PK") {
        "ZIP/DOCX/etc"
    } else {
        "Sconosciuto"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Carica credenziali dalle variabili d'ambiente
    let username = env::var("SPAGGIARI_USERNAME")
        .map_err(|_| "Variabile d'ambiente SPAGGIARI_USERNAME non impostata")?;
    let password = env::var("SPAGGIARI_PASSWORD")
        .map_err(|_| "Variabile d'ambiente SPAGGIARI_PASSWORD non impostata")?;

    println!("🔐 Effettuo il login...");
    let session = SpaggiariSession::new(&username, &password).await?;

    println!("✅ Sessione valida!");

    // Ottieni la bacheca
    let bacheca = session.get_bacheca().await?;
    println!("📄 Trovate {} comunicazioni lette", bacheca.read.len());

    // Prendi la prima comunicazione come esempio
    if let Some(circolare) = bacheca.read.first() {
        println!(
            "\n📄 Elaborando comunicazione: {} (Codice: {})",
            circolare.titolo, circolare.codice
        );

        // Ottieni i dettagli della comunicazione
        let comunicazione = session.get_comunicazione(&circolare.id).await?;

        println!("\n=== Metodo 1: Scarica allegati singolarmente ===");
        // Scarica ogni allegato in memoria uno alla volta
        for allegato in &comunicazione.allegati {
            let download_url = format!(
                "https://web.spaggiari.eu/sif/app/default/bacheca_personale.php?action=file_download&com_id={}",
                allegato.allegato_id
            );

            println!("\n📎 Scaricando allegato ID: {}", allegato.allegato_id);

            // Usa la nuova funzione per ottenere il contenuto binario
            let (filename, content) = session.download_file_bytes(&download_url).await?;

            println!("✅ File scaricato: {} ({} bytes)", filename, content.len());
            println!("   Tipo di file: {}", detect_file_type(&content));
        }

        println!("\n=== Metodo 2: Scarica tutti gli allegati in una volta ===");
        // Usa la nuova funzione per scaricare tutti gli allegati
        let files = session
            .download_allegati_bytes(&comunicazione.allegati)
            .await?;

        println!("📦 Scaricati {} allegati in memoria", files.len());
        for (filename, content) in files {
            println!(
                "✅ {}: {} bytes ({})",
                filename,
                content.len(),
                detect_file_type(&content)
            );

            // Esempio: salva il file
            let output_path = format!("download_bytes/{}", filename);
            std::fs::create_dir_all("download_bytes")?;
            std::fs::write(&output_path, &content)?;
            println!("💾 File salvato in: {}", output_path);
        }
    }

    Ok(())
}

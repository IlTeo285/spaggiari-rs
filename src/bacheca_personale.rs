use anyhow;
use csv::Writer;
use regex::Regex;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::fs::File;
use std::io::copy;

const url_bacheca: &str = "https://web.spaggiari.eu/sif/app/default/bacheca_personale.php";
const url_comunicazioni: &str =
    "https://web.spaggiari.eu/sif/app/default/bacheca_comunicazione.php";

#[derive(Debug, Clone, Deserialize)]
pub struct Circolare {
    pub id: String,
    pub codice: i32,
    pub titolo: String,
    pub testo: String,
    #[serde(rename = "data_start")]
    pub data_start: String,
    #[serde(rename = "data_stop")]
    pub data_stop: String,
    #[serde(rename = "tipo_com")]
    pub tipo_com: String,
    #[serde(rename = "tipo_com_filtro")]
    pub tipo_com_filtro: String,
    #[serde(rename = "tipo_com_desc")]
    pub tipo_com_desc: String,
    #[serde(rename = "nome_file")]
    pub nome_file: Option<String>,
    pub richieste: String,
    #[serde(rename = "id_relazione")]
    pub id_relazione: String,
    #[serde(rename = "conf_lettura")]
    pub conf_lettura: String,
    #[serde(rename = "flag_risp")]
    pub flag_risp: String,
    #[serde(rename = "testo_risp")]
    pub testo_risp: Option<String>,
    #[serde(rename = "file_risp")]
    pub file_risp: Option<String>,
    #[serde(rename = "flag_accettazione")]
    pub flag_accettazione: String,
    pub modificato: String,
    #[serde(rename = "evento_data")]
    pub evento_data: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Bacheca {
    pub read: Vec<Circolare>,
    pub msg_new: Option<Vec<Circolare>>,
}

// Nuova funzione per scrivere la bacheca su CSV (solo in modalità debug)
fn write_bacheca_to_csv(bacheca: &Bacheca) -> Result<(), anyhow::Error> {
    if cfg!(debug_assertions) {
        let mut wtr = Writer::from_writer(File::create("bacheca.csv")?);
        // Scrivi header con tutti i campi di Circolare
        wtr.write_record(&[
            "tipo",
            "id",
            "codice",
            "titolo",
            "testo",
            "data_start",
            "data_stop",
            "tipo_com",
            "tipo_com_filtro",
            "tipo_com_desc",
            "nome_file",
            "richieste",
            "id_relazione",
            "conf_lettura",
            "flag_risp",
            "testo_risp",
            "file_risp",
            "flag_accettazione",
            "modificato",
            "evento_data",
        ])?;

        // Scrivi righe per "read"
        for circolare in &bacheca.read {
            wtr.write_record(&[
                "read",
                &circolare.id,
                &circolare.codice.to_string(),
                &circolare.titolo,
                &circolare.testo,
                &circolare.data_start,
                &circolare.data_stop,
                &circolare.tipo_com,
                &circolare.tipo_com_filtro,
                &circolare.tipo_com_desc,
                &circolare.nome_file.as_deref().unwrap_or(""),
                &circolare.richieste,
                &circolare.id_relazione,
                &circolare.conf_lettura,
                &circolare.flag_risp,
                &circolare.testo_risp.as_deref().unwrap_or(""),
                &circolare.file_risp.as_deref().unwrap_or(""),
                &circolare.flag_accettazione,
                &circolare.modificato,
                &circolare.evento_data,
            ])?;
        }

        // Scrivi righe per "msg_new" solo se presente
        if let Some(msg_new_vec) = &bacheca.msg_new {
            for circolare in msg_new_vec {
                wtr.write_record(&[
                    "msg_new",
                    &circolare.id,
                    &circolare.codice.to_string(),
                    &circolare.titolo,
                    &circolare.testo,
                    &circolare.data_start,
                    &circolare.data_stop,
                    &circolare.tipo_com,
                    &circolare.tipo_com_filtro,
                    &circolare.tipo_com_desc,
                    &circolare.nome_file.as_deref().unwrap_or(""),
                    &circolare.richieste,
                    &circolare.id_relazione,
                    &circolare.conf_lettura,
                    &circolare.flag_risp,
                    &circolare.testo_risp.as_deref().unwrap_or(""),
                    &circolare.file_risp.as_deref().unwrap_or(""),
                    &circolare.flag_accettazione,
                    &circolare.modificato,
                    &circolare.evento_data,
                ])?;
            }
        }

        wtr.flush()?;
        println!("💾 Bacheca salvata su bacheca.csv (modalità debug)");
    }
    Ok(())
}

// Nuova funzione per estrarre comunicazione_id e allegato_id dai tag <a class="dwl_allegato">
pub fn extract_allegati(html: &str) -> Result<Vec<(String, String)>, anyhow::Error> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a.dwl_allegato")
        .map_err(|e| anyhow::anyhow!("Errore nel parsing del selettore: {}", e))?;

    let mut allegati = Vec::new();
    for element in document.select(&selector) {
        let comunicazione_id = element
            .value()
            .attr("comunicazione_id")
            .unwrap_or("")
            .to_string();
        let allegato_id = element
            .value()
            .attr("allegato_id")
            .unwrap_or("")
            .to_string();
        allegati.push((comunicazione_id, allegato_id));
    }

    Ok(allegati)
}

// Nuova funzione per scaricare un file da un URL
pub fn download_file(
    client: &Client,
    url: &str,
    session_id: &str,
    destination_path: &str,
) -> Result<String, anyhow::Error> {
    let mut response = client
        .get(url)
        .header(
            "Cookie",
            format!("PHPSESSID={}; webidentity=G13070983V", session_id),
        ) //TODO get from args
        .send()?;

    if response.status().is_success() {
        // Estrai filename da Content-Disposition
        let content_disposition = response
            .headers()
            .get("content-disposition")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let filename = extract_filename_from_disposition(content_disposition)
            .unwrap_or_else(|| "file_sconosciuto".to_string());

        let filepath = format!("{}/{}", destination_path, filename); // destination_path è una directory, aggiungi il filename
        // Assicurati che la directory esista
        if let Some(parent) = std::path::Path::new(&filepath).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&filepath)?;
        copy(&mut response, &mut file)?;
        println!("📥 File scaricato: {}", filepath);
        Ok(filepath)
    } else {
        println!(
            "❌ Download fallito per {}: Status {}",
            url,
            response.status()
        );
        Err(anyhow::anyhow!("Download fallito: {}", response.status()))
    }
}

// Funzione helper per estrarre il filename da Content-Disposition
fn extract_filename_from_disposition(disposition: &str) -> Option<String> {
    let re = Regex::new(r#"filename=([^;]+)"#).ok()?;
    re.captures(disposition)?
        .get(1)?
        .as_str()
        .trim_matches('"') // Rimuovi eventuali virgolette
        .to_string()
        .into()
}

// Nuova funzione per scaricare tutti gli allegati
pub fn download_allegati(
    client: &Client,
    session_id: &str,
    allegati: &[Allegato],
    destination_path: &str,
) -> Result<(), anyhow::Error> {
    for allegato in allegati {
        let download_url = format!(
            "https://web.spaggiari.eu/sif/app/default/bacheca_personale.php?action=file_download&com_id={}",
            allegato.allegato_id
        );
        download_file(client, &download_url, session_id, destination_path)?;
    }
    Ok(())
}

pub fn get_backeca(client: &Client, session_id: &str) -> Result<Bacheca, anyhow::Error> {
    let response = client
        .get(url_bacheca)
        .query(&[("action", "get_comunicazioni"), ("ncna", "1")]) // Aggiunti i form data come query parameters
        .header(
            "Cookie",
            format!("PHPSESSID={}; webidentity=G13070983V", session_id),
        ) //TODO get from args
        .send()?;

    let status = response.status();

    println!("📊 Risposta bacheca - Status: {}", status);

    if status.is_success() {
        let text = response.text()?;
        //println!("{}", text);
        match serde_json::from_str::<Bacheca>(&text) {
            Ok(bacheca) => {
                // Chiama la funzione separata per scrivere il CSV
                write_bacheca_to_csv(&bacheca)?;
                Ok(bacheca)
            }
            Err(e) => {
                println!("Deserialize error {}", e.to_string());
                Err(e.into())
            }
        }
    } else {
        println!("❌ Il token non sembra funzionare. Status: {}", status);
        Err(anyhow::anyhow!("Il token non sembra funzionare"))
    }
}

// Nuova funzione per estrarre il testo dalla comunicazione
pub fn extract_testo_comunicazione(html: &str) -> Result<String, anyhow::Error> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("div.comunicazione_testo")
        .map_err(|e| anyhow::anyhow!("Errore nel parsing del selettore: {}", e))?;

    if let Some(element) = document.select(&selector).next() {
        let testo = element.text().collect::<Vec<_>>().join(" ");
        Ok(testo)
    } else {
        Ok("".to_string()) // Se non trovato, restituisci stringa vuota
    }
}

pub struct Allegato {
    pub comunicazione_id: String,
    pub allegato_id: String,
}

pub struct Comunicazione {
    pub testo: String,
    pub allegati: Vec<Allegato>,
}

pub fn get_comunicazioni(
    client: &Client,
    session_id: &str,
    comm_id: &str,
) -> Result<Comunicazione, anyhow::Error> {
    let response = client
        .get(url_comunicazioni)
        .query(&[("action", "risposta_com"), ("com_id", comm_id)]) // Aggiunti i form data come query parameters
        .header(
            "Cookie",
            format!("PHPSESSID={}; webidentity=G13070983V", session_id),
        ) //TODO get from args
        .send()?;

    let status = response.status();

    println!("📊 Risposta bacheca - Status: {}", status);

    if status.is_success() {
        let text = response.text()?;
        //println!("{}", text);

        // Estrai gli allegati dal body HTML
        let allegati = extract_allegati(&text)?;

        // Estrai il testo della comunicazione
        let testo = extract_testo_comunicazione(&text)?;
        println!("📝 Testo comunicazione: {}", testo);

        Ok(Comunicazione {
            testo,
            allegati: allegati
                .into_iter()
                .map(|(com_id, all_id)| Allegato {
                    comunicazione_id: com_id,
                    allegato_id: all_id,
                })
                .collect(),
        })
    } else {
        println!("❌ Il token non sembra funzionare. Status: {}", status);
        Err(anyhow::anyhow!("Il token non sembra funzionare"))
    }
}

//! # Spaggiari RS
//!
//! Una libreria Rust per interagire con il portale Spaggiari (Registro Elettronico).
//!
//! Questa libreria fornisce funzionalità per:
//! - Effettuare il login al portale Spaggiari
//! - Ottenere la bacheca personale
//! - Scaricare comunicazioni e allegati
//! - Gestire i token di sessione

pub mod bacheca_personale;
pub mod error;
pub mod login;

use reqwest::cookie::Jar;
use reqwest::Client;
use std::sync::Arc;

// Re-export delle strutture principali
pub use bacheca_personale::{download_allegati, download_allegati_bytes, download_file, download_file_bytes, get_backeca, get_comunicazioni, Allegato, Bacheca, Circolare, Comunicazione};
pub use error::SpaggiariError;
pub use login::{login, test_session_token, AccountInfo, Auth, LoginResponse};

/// Crea un client HTTP configurato per Spaggiari
///
/// # Returns
///
/// Un `Client` reqwest configurato con cookies e user agent appropriati
///
/// # Example
///
/// ```
/// use spaggiari_rs::create_client;
///
/// let client = create_client().unwrap();
/// ```
pub fn create_client() -> Result<Client, reqwest::Error> {
    let jar = Jar::default();
    let jar = Arc::new(jar);

    Client::builder().cookie_provider(jar).user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)").build()
}

/// Struttura per gestire una sessione Spaggiari
///
/// Contiene il client HTTP e il token di sessione necessari
/// per effettuare le chiamate API
pub struct SpaggiariSession {
    pub client: Client,
    pub session_token: String,
    identity: String,
}

impl SpaggiariSession {
    /// Crea una nuova sessione effettuando il login
    ///
    /// # Arguments
    ///
    /// * `username` - Il codice fiscale dell'utente
    /// * `password` - La password dell'utente
    ///
    /// # Returns
    ///
    /// Una `SpaggiariSession` autenticata
    ///
    /// # Example
    ///
    /// ```
    /// use spaggiari_rs::SpaggiariSession;
    ///
    /// let session = SpaggiariSession::new("CODICE_FISCALE", "PASSWORD").unwrap();
    /// ```
    pub async fn new(username: &str, password: &str) -> Result<Self, SpaggiariError> {
        let client = create_client()?;
        let session_token = login(&client, username, password).await?;

        Ok(SpaggiariSession {
            client,
            session_token,
            identity: username.to_string(),
        })
    }

    /// Crea una sessione usando un token esistente
    ///
    /// # Arguments
    ///
    /// * `session_token` - Il token di sessione salvato
    ///
    /// # Returns
    ///
    /// Una `SpaggiariSession` se il token è valido
    ///
    /// # Example
    ///
    /// ```
    /// use spaggiari_rs::SpaggiariSession;
    ///
    /// let session = SpaggiariSession::from_token("token_esistente").unwrap();
    /// ```
    pub async fn from_token(session_token: String) -> Result<Self, SpaggiariError> {
        let client = create_client()?;
        let username = std::env::var("SPAGGIARI_USERNAME")?;
        // Verifica che il token sia valido
        if !test_session_token(&client, &session_token, &username).await? {
            return Err(SpaggiariError::InvalidSessionToken);
        }

        Ok(SpaggiariSession {
            client,
            session_token,
            identity: username,
        })
    }

    /// Verifica se il token di sessione è ancora valido
    ///
    /// # Returns
    ///
    /// `true` se il token è valido, `false` altrimenti
    pub async fn is_valid(&self) -> Result<bool, SpaggiariError> {
        test_session_token(&self.client, &self.session_token, &self.identity).await
    }

    /// Ottiene la bacheca personale
    ///
    /// # Returns
    ///
    /// La struttura `Bacheca` contenente tutte le comunicazioni
    ///
    /// # Example
    ///
    /// ```
    /// let session = SpaggiariSession::new("username", "password").unwrap();
    /// let bacheca = session.get_bacheca().unwrap();
    /// println!("Comunicazioni lette: {}", bacheca.read.len());
    /// ```
    pub async fn get_bacheca(&self) -> Result<Bacheca, SpaggiariError> {
        Ok(get_backeca(&self.client, &self.session_token, &self.identity).await?)
    }

    /// Ottiene una comunicazione specifica
    ///
    /// # Arguments
    ///
    /// * `circolare_id` - L'ID della circolare da ottenere
    ///
    /// # Returns
    ///
    /// La struttura `Comunicazione` con tutti i dettagli
    pub async fn get_comunicazione(&self, circolare_id: &str) -> Result<Comunicazione, SpaggiariError> {
        Ok(get_comunicazioni(&self.client, &self.session_token, circolare_id, "").await?)
    }

    /// Scarica tutti gli allegati di una comunicazione
    ///
    /// # Arguments
    ///
    /// * `allegati` - Lista degli allegati da scaricare
    /// * `folder_path` - Percorso della cartella dove salvare i file
    pub async fn download_allegati(&self, allegati: &[Allegato], folder_path: &str) -> Result<(), SpaggiariError> {
        Ok(download_allegati(&self.client, &self.session_token, allegati, folder_path).await?)
    }

    /// Scarica un file e ritorna il contenuto binario
    ///
    /// # Arguments
    ///
    /// * `url` - URL del file da scaricare
    ///
    /// # Returns
    ///
    /// Una tupla contenente il nome del file e il contenuto binario
    ///
    /// # Example
    ///
    /// ```
    /// let (filename, content) = session.download_file_bytes("https://...").await?;
    /// println!("Scaricato {} ({} bytes)", filename, content.len());
    /// ```
    pub async fn download_file_bytes(&self, url: &str) -> Result<(String, Vec<u8>), SpaggiariError> {
        Ok(download_file_bytes(&self.client, url, &self.session_token).await?)
    }

    /// Scarica tutti gli allegati in memoria e ritorna un vettore di risultati
    ///
    /// # Arguments
    ///
    /// * `allegati` - Lista degli allegati da scaricare
    ///
    /// # Returns
    ///
    /// Un vettore di tuple contenenti il nome del file e il contenuto binario
    ///
    /// # Example
    ///
    /// ```
    /// let comunicazione = session.get_comunicazione("123").await?;
    /// let files = session.download_allegati_bytes(&comunicazione.allegati).await?;
    /// for (filename, content) in files {
    ///     println!("Scaricato {} ({} bytes)", filename, content.len());
    /// }
    /// ```
    pub async fn download_allegati_bytes(&self, allegati: Vec<Allegato>) -> Result<Vec<(String, Vec<u8>)>, SpaggiariError> {
        Ok(download_allegati_bytes(&self.client, &self.session_token, allegati).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_client() {
        let client = create_client();
        assert!(client.is_ok());
    }
}

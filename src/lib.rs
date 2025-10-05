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
pub mod login;

use reqwest::blocking::Client;
use reqwest::cookie::Jar;
use std::sync::Arc;

// Re-export delle strutture principali
pub use bacheca_personale::{
    download_allegati, download_file, get_backeca, get_comunicazioni, Allegato, Bacheca, Circolare,
    Comunicazione,
};
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

    Client::builder()
        .cookie_provider(jar)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .build()
}

/// Struttura per gestire una sessione Spaggiari
///
/// Contiene il client HTTP e il token di sessione necessari
/// per effettuare le chiamate API
pub struct SpaggiariSession {
    pub client: Client,
    pub session_token: String,
    identity: String
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
    pub fn new(username: &str, password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = create_client()?;
        let session_token = login(&client, username, password)?;

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
    pub fn from_token(session_token: String) -> Result<Self, Box<dyn std::error::Error>> {
        let client = create_client()?;
        let username = std::env::var("SPAGGIARI_USERNAME")?;
        // Verifica che il token sia valido
        if !test_session_token(&client, &session_token, &username)? {
            return Err("Token di sessione non valido".into());
        }

        Ok(SpaggiariSession {
            client,
            session_token,
            identity: username
        })
    }

    /// Verifica se il token di sessione è ancora valido
    ///
    /// # Returns
    ///
    /// `true` se il token è valido, `false` altrimenti
    pub fn is_valid(&self) -> Result<bool, Box<dyn std::error::Error>> {
        test_session_token(&self.client, &self.session_token, &self.identity)
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
    pub fn get_bacheca(&self) -> Result<Bacheca, Box<dyn std::error::Error>> {
        Ok(get_backeca(&self.client, &self.session_token, &self.identity)?)
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
    pub fn get_comunicazione(
        &self,
        circolare_id: &str,
    ) -> Result<Comunicazione, Box<dyn std::error::Error>> {
        Ok(get_comunicazioni(
            &self.client,
            &self.session_token,
            circolare_id,
            ""
        )?)
    }

    /// Scarica tutti gli allegati di una comunicazione
    ///
    /// # Arguments
    ///
    /// * `allegati` - Lista degli allegati da scaricare
    /// * `folder_path` - Percorso della cartella dove salvare i file
    pub fn download_allegati(
        &self,
        allegati: &[Allegato],
        folder_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(download_allegati(
            &self.client,
            &self.session_token,
            allegati,
            folder_path,
        )?)
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

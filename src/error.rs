use thiserror::Error;

/// Errori specifici per la libreria spaggiari-rs
#[derive(Debug, Error)]
pub enum SpaggiariError {
    /// Errore di autenticazione - credenziali non valide
    #[error("Autenticazione fallita: credenziali non valide")]
    AuthenticationFailed,

    /// Token di sessione non valido o scaduto
    #[error("Token di sessione non valido o scaduto")]
    InvalidSessionToken,

    /// Errore nella richiesta HTTP
    #[error("Errore richiesta HTTP: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Errore nella deserializzazione JSON
    #[error("Errore deserializzazione JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Errore API - risposta inaspettata dal server
    #[error("Errore API: {message}")]
    ApiError { message: String },

    /// Errore variabile d'ambiente mancante
    #[error("Variabile d'ambiente mancante: {0}")]
    EnvVarError(#[from] std::env::VarError),

    /// Errore I/O durante il salvataggio dei file
    #[error("Errore I/O: {0}")]
    IoError(#[from] std::io::Error),

    /// Comunicazione non trovata
    #[error("Comunicazione con ID '{0}' non trovata")]
    ComunicazioneNotFound(String),

    /// Allegato non trovato
    #[error("Allegato '{0}' non trovato")]
    AllegatoNotFound(String),

    /// Errore nel parsing della risposta
    #[error("Errore parsing risposta: {details}")]
    ParseError { details: String },

    /// URL non valido o mancante
    #[error("URL non valido: {0}")]
    InvalidUrl(String),

    /// Errore di rete generico
    #[error("Errore di rete: {0}")]
    NetworkError(String),

    /// Errore generico
    #[error("Errore generico: {0}")]
    Generic(String),
}

// Conversione da stringhe per compatibilità
impl From<String> for SpaggiariError {
    fn from(s: String) -> Self {
        SpaggiariError::Generic(s)
    }
}

impl From<&str> for SpaggiariError {
    fn from(s: &str) -> Self {
        SpaggiariError::Generic(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SpaggiariError::AuthenticationFailed;
        assert_eq!(err.to_string(), "Autenticazione fallita: credenziali non valide");

        let err = SpaggiariError::InvalidSessionToken;
        assert_eq!(err.to_string(), "Token di sessione non valido o scaduto");

        let err = SpaggiariError::ComunicazioneNotFound("123".to_string());
        assert_eq!(err.to_string(), "Comunicazione con ID '123' non trovata");
    }

    #[test]
    fn test_error_from_string() {
        let err: SpaggiariError = "test error".into();
        assert_eq!(err.to_string(), "Errore generico: test error");
    }
}

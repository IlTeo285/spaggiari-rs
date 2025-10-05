# Spaggiari RS

Una libreria Rust per interagire con il portale Spaggiari (Registro Elettronico).

## Caratteristiche

- ✅ Login al portale Spaggiari
- ✅ Gestione sessioni e token
- ✅ Accesso alla bacheca personale
- ✅ Download delle comunicazioni
- ✅ Download degli allegati
- ✅ Gestione automatica dei cookies

## Installazione

Aggiungi questa libreria al tuo `Cargo.toml`:

```toml
[dependencies]
spaggiari-rs = "0.1.0"
```

## Configurazione

Prima di utilizzare la libreria, devi impostare le seguenti variabili d'ambiente:

```bash
export SPAGGIARI_USERNAME="tuo_codice_fiscale"
export SPAGGIARI_PASSWORD="tua_password"
```

### Configurazione rapida

1. Copia il file di esempio:

```bash
cp .env.example .env
```

2. Modifica `.env` con le tue credenziali reali

3. Carica le variabili d'ambiente:

```bash
source .env
```

### Su Linux/macOS

```bash
echo 'export SPAGGIARI_USERNAME="tuo_codice_fiscale"' >> ~/.bashrc
echo 'export SPAGGIARI_PASSWORD="tua_password"' >> ~/.bashrc
source ~/.bashrc
```

### Su Windows

```cmd
set SPAGGIARI_USERNAME=tuo_codice_fiscale
set SPAGGIARI_PASSWORD=tua_password
```

### Su Windows (variabili permanenti)

```cmd
setx SPAGGIARI_USERNAME "tuo_codice_fiscale"
setx SPAGGIARI_PASSWORD "tua_password"
```

## Utilizzo

### Esempio base

````rust
use spaggiari_rs::SpaggiariSession;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Carica credenziali dalle variabili d'ambiente
    let username = env::var("SPAGGIARI_USERNAME")?;
    let password = env::var("SPAGGIARI_PASSWORD")?;

    // Crea una nuova sessione con login
    let session = SpaggiariSession::new(&username, &password)?;

    // Ottieni la bacheca
    let bacheca = session.get_bacheca()?;
    println!("Trovate {} comunicazioni", bacheca.read.len());

    Ok(())
}
```### Uso di un token esistente

```rust
use spaggiari_rs::SpaggiariSession;

// Se hai già un token salvato
let session = SpaggiariSession::from_token("token_esistente".to_string())?;

// Verifica se è ancora valido
if session.is_valid()? {
    println!("Token valido!");
    let bacheca = session.get_bacheca()?;
    // ... usa la bacheca
}
````

### Configurazione manuale del client

```rust
use spaggiari_rs::{create_client, login::login, bacheca_personale::get_backeca};

let client = create_client()?;
let token = login(&client, "username", "password")?;
let bacheca = get_backeca(&client, &token)?;
```

## API

### SpaggiariSession

La struttura principale per gestire una sessione autenticata.

#### Metodi

- `new(username, password)` - Crea una nuova sessione con login
- `from_token(token)` - Crea una sessione da un token esistente
- `is_valid()` - Verifica se il token è ancora valido
- `get_bacheca()` - Ottiene la bacheca personale
- `get_comunicazione(id)` - Ottiene una comunicazione specifica
- `download_allegati(allegati, folder)` - Scarica gli allegati

### Strutture dati

#### Bacheca

```rust
pub struct Bacheca {
    pub read: Vec<Circolare>,
    pub msg_new: Option<Vec<Circolare>>,
}
```

#### Circolare

```rust
pub struct Circolare {
    pub id: String,
    pub codice: i32,
    pub titolo: String,
    pub data_start: String,
    pub data_stop: String,
    // ... altri campi
}
```

#### Comunicazione

```rust
pub struct Comunicazione {
    pub id: String,
    pub testo: String,
    pub allegati: Vec<Allegato>,
    // ... altri campi
}
```

## Esempi

Guarda la cartella `examples/` per esempi completi di utilizzo.

## CLI

Il progetto include anche un'applicazione CLI che puoi eseguire con:

```bash
cargo run --bin spaggiari-cli
```

## Compilazione

```bash
# Compila la libreria
cargo build

# Esegui i test
cargo test

# Compila la CLI
cargo build --bin spaggiari-cli
```

## Licenza

MIT

## Contributi

I contributi sono benvenuti! Apri pure issue e pull request.

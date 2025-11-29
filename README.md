# Spaggiari RS

Una libreria Rust e CLI per interagire con il portale Spaggiari (Registro Elettronico).

## Caratteristiche

- ✅ Login al portale Spaggiari
- ✅ Gestione sessioni e token (salvataggio automatico)
- ✅ Accesso alla bacheca personale
- ✅ Elenco circolari (lette e nuove)
- ✅ Visualizzazione dettagli circolari
- ✅ Download delle comunicazioni e degli allegati
- ✅ Supporto file `.env` per le credenziali

## CLI (Command Line Interface)

Il progetto include un potente strumento da riga di comando per interagire con il registro elettronico.

### Installazione ed Esecuzione

Puoi eseguire la CLI direttamente tramite `cargo`:

```bash
cargo run -- <COMANDO> [OPZIONI]
```

Oppure compilarla:

```bash
cargo build --release --bin spaggiari-cli
./target/release/spaggiari-cli <COMANDO>
```

### Configurazione

La CLI supporta il caricamento delle credenziali da un file `.env` nella directory corrente.

Crea un file `.env`:
```env
SPAGGIARI_USERNAME=tuo_codice
SPAGGIARI_PASSWORD=tua_password
```

In alternativa, puoi passare le credenziali direttamente al comando `login` o impostare le variabili d'ambiente nel tuo sistema.

### Comandi Disponibili

#### 1. Login
Effettua il login e salva il token di sessione localmente (`phpsessid.token`).

```bash
# Usa credenziali da .env o variabili d'ambiente
cargo run -- login

# Oppure passa le credenziali esplicitamente
cargo run -- login --username <USER> --password <PASS>
```

#### 2. Verifica Token
Controlla se il token salvato è ancora valido.

```bash
cargo run -- check-token
```

#### 3. Elenco Circolari
Mostra l'elenco delle circolari presenti in bacheca (sia nuove che lette) con i relativi ID.

```bash
cargo run -- list
```
*Output esempio:*
```text
📋 Elenco Circolari:
---------------------------------------------------
🆕 ID: 12345 - Circolare Importante
✅ ID: 67890 - Orario Lezioni
---------------------------------------------------
```

#### 4. Dettagli Circolare
Mostra il testo completo e gli allegati di una specifica circolare. Richiede l'ID della circolare (ottenibile con `list`).

```bash
cargo run -- details --code <ID_CIRCOLARE>
```

#### 5. Download
Scarica tutte le comunicazioni e i relativi allegati nella cartella `download/`.

```bash
cargo run -- download
```
Verrà creata una struttura di cartelle organizzata per codice circolare.

---

## Utilizzo come Libreria Rust

Puoi usare `spaggiari-rs` anche come libreria nel tuo progetto Rust.

### Installazione

Aggiungi al tuo `Cargo.toml`:

```toml
[dependencies]
spaggiari-rs = { git = "https://github.com/IlTeo285/spaggiari-rs" }
```

### Esempio di utilizzo

```rust
use spaggiari_rs::SpaggiariSession;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Login
    let username = env::var("SPAGGIARI_USERNAME")?;
    let password = env::var("SPAGGIARI_PASSWORD")?;
    let session = SpaggiariSession::new(&username, &password).await?;

    // 2. Ottieni la bacheca
    let bacheca = session.get_bacheca().await?;
    println!("Trovate {} comunicazioni lette", bacheca.read.len());

    // 3. Leggi una comunicazione specifica
    if let Some(circolare) = bacheca.read.first() {
        let dettagli = session.get_comunicazione(&circolare.id).await?;
        println!("Testo: {}", dettagli.testo);
    }

    Ok(())
}
```

## Sviluppo

### Requisiti
- Rust (latest stable)
- OpenSSL (`libssl-dev` su Ubuntu/Debian)

### Comandi utili

```bash
# Compila tutto
cargo build

# Esegui i test
cargo test

# Check del codice
cargo check
```

## Licenza

MIT

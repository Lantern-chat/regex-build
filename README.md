regex-build
===========

Utility for precompiled regex-automata regexes in build scripts.

### `Cargo.toml`

```toml
[dependencies]
regex-automata = { version = "0.1.10", default_features = false }
once_cell = "1.17.1"
regex-build = { git = "https://github.com/Lantern-chat/regex-build" }

[build-dependencies]
regex-build = { git = "https://github.com/Lantern-chat/regex-build", features = ["build"] }
```

### `build.rs`

```rust
use std::{env, fs::File, io::BufWriter, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(&env::var("OUT_DIR")?).join("codegen.rs");
    let mut file = BufWriter::new(File::create(path)?);

    regex_build::write_regex(
        "ATTRIBUTE_RE", // helps with splitting name="value"
        r#"[a-zA-Z_][0-9a-zA-Z\-_]+\s*=\s*(
            ("(?:\\"|[^"])*[^\\]")| # name="value"
            ('(?:\\'|[^'])*[^\\]')| # name='value'
            ([^'"](?:\\\s|[^\s>]*)) # name=value or name=value>
        )"#,
        &mut file,
    )?;

    Ok(())
}
```

### `lib.rs`
Or anywhere else

```rust
pub mod regexes {
    use regex_automata::{DenseDFA, Regex};
    include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
}
```
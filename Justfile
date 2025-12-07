set windows-shell := ["C:\\Program Files\\Git\\bin\\sh.exe","-c"]

firmware:
    cargo --config firmware/.cargo/config.toml run -p firmware --profile firmware

server:
    cargo run -p server

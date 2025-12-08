set windows-shell := ["C:\\Program Files\\Git\\bin\\sh.exe","-c"]

firmware:
    cargo --config firmware/.cargo/config.toml run --release --manifest-path firmware/Cargo.toml

build-firmware:
    cargo --config firmware/.cargo/config.toml build --release --manifest-path firmware/Cargo.toml

server:
    cargo run --manifest-path server/Cargo.toml

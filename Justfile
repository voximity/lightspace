set windows-shell := ["C:\\Program Files\\Git\\bin\\sh.exe", "-c"]

_fw ov mcu:
    cargo {{ ov }} \
        --config firmware/.cargo/config.toml \
        run --release \
        --manifest-path firmware/Cargo.toml \
        --no-default-features \
        --features {{ mcu }}

_fw-offline ov mcu:
    cargo {{ ov }} \
        --config firmware/.cargo/config.toml \
        run --release \
        --manifest-path firmware/Cargo.toml \
        --no-default-features \
        --features {{ mcu }},offline

esp32c6:
    @just _fw +stable esp32c6

esp32c6-offline:
    @just _fw-offline +stable esp32c6

esp32s3:
    @just _fw +esp esp32s3

esp32s3-offline:
    @just _fw-offline +esp esp32s3

server:
    cargo run --manifest-path server/Cargo.toml

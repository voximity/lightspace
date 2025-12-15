# lightspace

ESP32-C6-based (with some support for the ESP32-S3) WS2812B LED strip installation
project - heavily WIP, tons of hard-coded garbage

End goal: spatially-defined, network-controlled LEDs driven by a master server that
connects to any number of ESP32 nodes and coordinates application-driven effects

## Upload and run firmware

```
cargo install just
just esp32c6
```

To run without Wi-Fi features,

```
just esp32c6-offline
```

## Run server

```
just server
```

## Using the ESP32-S3

1. Switch the `targets` and `channel` in [firmware/rust-toolchain.toml](./firmware/rust-toolchain.toml)
2. Set `default = ["esp32c6"]` to `default = []` in [firmware/Cargo.toml](./firmware/Cargo.toml)
3. Switch the `target` in `[build]` and `cargo.features` in `[rust-analyzer]` in [firmware/.cargo/config.toml](./firmware/.cargo/config.toml)
4. Run with `just esp32s3` instead

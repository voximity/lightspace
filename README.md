# lightspace

ESP32-S3-based (with some support for the ESP32-C6) WS2812B LED strip installation
project - heavily WIP, tons of hard-coded garbage

End goal: spatially-defined, network-controlled LEDs driven by a master server that
connects to any number of ESP32 nodes and coordinates application-driven effects

## Upload and run firmware

```
cargo install just
just esp32s3
```

To run without Wi-Fi features,

```
just esp32s3-offline
```

## Run server

```
just server
```

## ESP32-C6

This project was originally written for the ESP32-C6, but I've switched to the ESP32-S3
to benefit from a second core. All signal transmission and LED effect computation is
pinned to the second core, while all Wi-Fi activity is pinned to the first core. This
reduces strange signal noise and flickering in my testing.

You can still use the ESP32-C6, but you will need to change some targets around
in `.cargo/config.toml` and `rust-toolchain.toml`, and use the `esp32c6` feature of `firmware`.

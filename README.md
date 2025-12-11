# lightspace

ESP32-C6-based WS2812B LED strip installation project - heavily WIP, tons of hard-coded garbage

End goal: spatially-defined, network-controlled LEDs driven by a master server that
connects to any number of ESP32 nodes and coordinates application-driven effects

## Upload and run firmware

```
cargo install just
just firmware
```

To run without Wi-Fi features,

```
just firmware-offline
```

## Run server

```
just server
```

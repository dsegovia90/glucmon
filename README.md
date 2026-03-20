# Glucmon

A system tray application that displays blood glucose readings from [Nightscout](https://nightscout.github.io/). Shows the current glucose value, trend direction, and time since last reading. The tray icon changes color based on severity (normal, concern, urgent).

Built with Tauri (Rust) and Vue 3.

## Setup

You need a Nightscout instance. Open the app, click Settings in the tray menu, and enter your Nightscout URL and API token. You can toggle between mg/dL and mmol/L.

## Building

Requires Rust, Node.js, and platform-specific dependencies (see [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)).

```
npm install
npm run tauri dev    # development
npm run tauri build  # production
```

## Platforms

macOS (ARM64, x86_64), Linux, Windows.

On macOS, the app runs as an accessory (tray only, no dock icon).

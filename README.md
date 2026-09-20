# Unnamed Enhancements

A focused Windows workspace for your mouse. Charcoal glass, a warm accent, and the controls that matter.

## Download

Download **UnnamedEnhancements.exe** from [Releases](https://github.com/7n8s/UnnamedEnhancements/releases/latest). Save it to a permanent folder and launch it directly; the release executable does not need ZIP extraction. Windows may show a security prompt for an unsigned app. Microsoft Edge WebView2 is required.

## What's inside

- Overview with connected-device status and quick DPI controls.
- Physical-button selection, X1 top/side views, and Windows side-button shortcuts.
- Typed DPI, a slider, and presets for the Attack Shark X1.
- Local profiles, import/export, and optional process-based profile switching.
- Mouse input tester and read-only HID diagnostics.
- Optional Serx assistant using a local Ollama installation.
- Custom colours, glass, images/GIFs, scaling, spacing, and reduced motion.

## Hardware boundaries

Attack Shark X1 has the existing hardware DPI implementation. Logitech G304/G305 and Glorious Model O Wired have detection/layout support, not verified native DPI control. Generic-device detection is opt-in.

Side-button shortcuts use a Windows hook while Unnamed runs and may affect other attached mice. Main buttons and the physical DPI button retain their native behavior. Polling, RGB and battery controls are deliberately absent because their protocols are not verified. Stored legacy settings remain intact; they are not presented as live hardware readings.

## Development

Install Node.js and the Rust/Tauri Windows prerequisites, then run:

```powershell
npm.cmd install
npm.cmd run tauri dev
npm.cmd run build:app
```

For hardware-free design testing, run `npm.cmd run dev` and open `http://localhost:1420/?demo=x1`. Use `demo=g305` or `demo=empty` for limited-support and disconnected states. This fixture is clearly labelled, mocks native commands, and is excluded from production.

Serx requires Ollama running locally and `ollama pull qwen2.5:3b-instruct`. It cannot change mouse settings for you.

# Unnamed Enhancements

A focused Windows workspace for your mouse, rebuilt as a compact command center with cooler glass, sharper typography, and device-first controls.

## Download

Download **UnnamedEnhancements.exe** from [Releases](https://github.com/7n8s/UnnamedEnhancements/releases/latest). Save it to a permanent folder and launch it directly; the release executable does not need ZIP extraction. Windows may show a security prompt for an unsigned app. Microsoft Edge WebView2 is required.

## What's inside

- Compact icon rail and a device-first overview with connected status and quick DPI controls.
- Physical-button selection, device-matched layouts, and Windows side-button shortcuts.
- Typed DPI, a slider, and presets for the Attack Shark X1 and Razer DeathAdder Essential.
- Local profiles with manual switching, import, and export.
- Mouse input tester and read-only HID diagnostics.
- Optional Serx assistant using a local Ollama installation.
- Custom colours, layered liquid glass, high-quality images/GIFs, scaling, spacing, and reduced motion.

## Hardware boundaries

Attack Shark X1 and Razer DeathAdder Essential (USB IDs `1532:006E`, `1532:0071`, and `1532:0098`) have hardware DPI control. Logitech G304/G305 and Glorious Model O Wired have detection/layout support without native DPI control. Generic-device detection is opt-in.

Side-button shortcuts use a Windows hook while Unnamed runs and may affect other attached mice. Main buttons and the physical DPI button retain their native behavior. Polling, RGB and battery controls are deliberately absent because their protocols are not verified. Stored legacy settings remain intact; they are not presented as live hardware readings.

## Development

Install Node.js and the Rust/Tauri Windows prerequisites, then run:

```powershell
npm.cmd install
npm.cmd run tauri dev
npm.cmd run build:app
```

For hardware-free design testing, run `npm.cmd run dev` and open `http://localhost:1420/?demo=x1`. Use `demo=deathadder`, `demo=g305`, or `demo=empty` for other supported and disconnected states. This fixture is clearly labelled, mocks native commands, and is excluded from production.

Serx requires Ollama running locally and `ollama pull qwen2.5:3b-instruct`. It cannot change mouse settings for you.

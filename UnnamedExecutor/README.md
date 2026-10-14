# Unnamed Executor — v0.2.0

A Windows desktop workspace for writing and running **standalone Luau** scripts locally.

> This project does not attach to Roblox, inject into Roblox processes, expose Roblox APIs, or attempt to evade anti-cheat detection.

## Features

- Dark UI with violet accents
- Multi-tab editor, open/save/save-as, and unsaved-change markers
- Luau syntax highlighting and line numbers
- Run current tab with the button or Ctrl+Enter
- Output console for print output and runtime errors
- CMake project that fetches Luau from upstream

## Requirements

- Windows 10/11
- Visual Studio 2022 with **Desktop development with C++**
- CMake 3.24+
- Git
- Qt 6.5+ for MSVC 2022 64-bit

## Build

Open **x64 Native Tools Command Prompt for VS 2022** and adjust the Qt path to match your installation:

    cmake -S . -B build -G "Visual Studio 17 2022" -A x64 -DCMAKE_PREFIX_PATH="C:/Qt/6.8.3/msvc2022_64"
    cmake --build build --config Release

Run `build/Release/UnnamedExecutor.exe`. The initial configure step needs internet access to fetch Luau. Qt runtime DLLs may need to be deployed beside the executable using Qt's `windeployqt` tool.

## Runtime limits

Scripts run in a local Luau VM. Roblox APIs such as `game`, `workspace`, `Instance`, and `Players` are not available.

## Status

This is an uncompiled starter source release; Windows build and runtime behavior have not yet been verified. For a more robust release, add a VM instruction/time budget, filesystem restrictions, automated tests, an app icon, and a signed installer.

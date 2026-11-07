# Latency-safe mouse input and Windows startup

Version 0.5.6 keeps the app completely out of the live mouse-input path unless a custom side-button shortcut actually needs it.

- Removes the always-on global mouse hook that could cause tiny cursor stalls in latency-sensitive games such as osu!.
- Installs the side-button hook only while a custom M4/M5 action is active, and removes it again when those actions are reset.
- Removes side-button customisation entirely for the GS Crush and guarantees that selecting it disables the global mouse hook.
- Gives the GS Crush a lean DPI-only interface: Overview, DPI, support, and app settings, with profiles, remapping, shortcut counters, and the mouse tester removed from its mode.
- Stops resending the saved DPI automatically when the app opens or merely detects the mouse; DPI writes now happen only after an explicit change or profile switch.
- Adds a **Start with Windows** option under Appearance → Application. Startup launches quietly in the system tray.
- Recognises the GS Crush through `1D57:A001` in 2.4 GHz receiver mode and `1D57:A011` in wired USB-C mode.
- Writes 50–22,000 DPI directly to the PAW3311/Beken onboard profile using the mouse's native feature-report format.
- Uses the correct packet length for receiver and wired connections, with serialized writes and the firmware's required settling delay.
- Prefers the writable Windows `COL04` feature-report collection and automatically tries every interface-2 collection before reporting a failure.
- Sends the GS Crush's native report ID `0x14` as an exact 60-byte packet through its `FF02/usage 2` vendor collection.
- Replaces the Attack Shark artwork with dedicated transparent top and side views of the black GS Crush.
- Updates Overview, DPI, device help, and supported-hardware wording for the GS Crush.
- Preserves the existing Attack Shark X1, DeathAdder Essential, SmartBuy RUSH Avatar, HyperX, Logitech, Glorious, and Apex 3 TKL support.

Download **UnnamedEnhancements.exe** below and open it directly. No ZIP extraction is needed for this release asset.

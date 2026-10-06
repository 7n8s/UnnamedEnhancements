# DEXP GS Crush native DPI protocol

Version 0.5.5 replaces the incompatible X11 DPI packet with the GS Crush's native protocol recovered from DEXP's official software.

- Recognises the GS Crush through `1D57:A001` in 2.4 GHz receiver mode and `1D57:A011` in wired USB-C mode.
- Writes 50–22,000 DPI directly to the PAW3311/Beken onboard profile using the mouse's native feature-report format.
- Uses the correct packet length for receiver and wired connections, with serialized writes and the firmware's required settling delay.
- Prefers the writable Windows `COL04` feature-report collection and automatically tries every interface-2 collection before reporting a failure.
- Sends the GS Crush's native report ID `0x14` as an exact 60-byte packet through its `FF02/usage 2` vendor collection.
- Replaces the Attack Shark artwork with dedicated transparent top and side views of the black GS Crush.
- Updates Overview, DPI, device help, and supported-hardware wording for the GS Crush.
- Preserves the existing Attack Shark X1, DeathAdder Essential, SmartBuy RUSH Avatar, HyperX, Logitech, Glorious, and Apex 3 TKL support.

Download **UnnamedEnhancements.exe** below and open it directly. No ZIP extraction is needed for this release asset.

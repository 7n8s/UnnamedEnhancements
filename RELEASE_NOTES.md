# DEXP GS Crush native Windows report padding

Version 0.5.4 fixes GS Crush DPI writes rejected by Windows with `os error 87`.

- Recognises the GS Crush through `1D57:A001` in 2.4 GHz receiver mode and `1D57:A011` in wired USB-C mode.
- Writes 50–22,000 DPI directly to the PAW3311/Beken onboard profile using the mouse's native feature-report format.
- Uses the correct packet length for receiver and wired connections, with serialized writes and the firmware's required settling delay.
- Prefers the writable Windows `COL04` feature-report collection and automatically tries every interface-2 collection before reporting a failure.
- Reads the device's native `FeatureReportByteLength` and zero-pads the 56-byte DPI packet exactly like DEXP's official HID library before calling `HidD_SetFeature`.
- Replaces the Attack Shark artwork with dedicated transparent top and side views of the black GS Crush.
- Updates Overview, DPI, device help, and supported-hardware wording for the GS Crush.
- Preserves the existing Attack Shark X1, DeathAdder Essential, SmartBuy RUSH Avatar, HyperX, Logitech, Glorious, and Apex 3 TKL support.

Download **UnnamedEnhancements.exe** below and open it directly. No ZIP extraction is needed for this release asset.

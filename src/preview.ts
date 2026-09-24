import { mockIPC } from "@tauri-apps/api/mocks";

/** Development-only visual QA. No USB access, downloads, or model requests. */
export function setupPreview() {
  const mode = new URLSearchParams(location.search).get("demo");
  const empty = mode === "empty";
  const logitech = mode === "g305";
  const deathadder = mode === "deathadder";
  const notice = document.createElement("div");
  notice.textContent = "Design preview · simulated device";
  notice.style.cssText = "position:fixed;bottom:3px;right:14px;z-index:100;font:10px system-ui;color:#b7a17b;pointer-events:none";
  document.body.append(notice);
  mockIPC(async (command, args) => {
    console.info("[preview IPC]", command, JSON.stringify(args));
    switch (command) {
      case "detect_mice": return empty ? [] : [{ id: "preview-device", name: deathadder ? "Razer DeathAdder Essential" : logitech ? "Logitech G305" : "Attack Shark X1", manufacturer: deathadder ? "Razer" : logitech ? "Logitech" : "Attack Shark", vid: deathadder ? "0x1532" : logitech ? "0x046d" : "0x3151", pid: deathadder ? "0x006e" : logitech ? "0xc53f" : "0x5031", connection: deathadder ? "Wired USB" : "USB", connected: true }];
      case "set_dpi": case "apply_button_mappings": case "set_minimize_to_tray": return;
      case "inspect_dpi_hardware": return [{ preview: true, interfaceNumber: 2, usagePage: "0xffff", note: "Simulated read-only diagnostic" }];
      case "test_button_action": return;
      default: throw new Error("Preview command not supported: " + command);
    }
  });
}

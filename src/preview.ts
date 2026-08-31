import { mockIPC } from "@tauri-apps/api/mocks";

/** Development-only visual QA. No USB access, downloads, or model requests. */
export function setupPreview() {
  const mode = new URLSearchParams(location.search).get("demo");
  const empty = mode === "empty";
  const logitech = mode === "g305";
  const notice = document.createElement("div");
  notice.textContent = "Design preview · simulated device";
  notice.style.cssText = "position:fixed;bottom:3px;right:14px;z-index:100;font:10px system-ui;color:#b7a17b;pointer-events:none";
  document.body.append(notice);
  mockIPC(async (command, args) => {
    console.info("[preview IPC]", command, JSON.stringify(args));
    switch (command) {
      case "detect_mice": return empty ? [] : [{ id: "preview-device", name: logitech ? "Logitech G305" : "Attack Shark X1", manufacturer: logitech ? "Logitech" : "Attack Shark", vid: logitech ? "0x046d" : "0x3151", pid: logitech ? "0xc53f" : "0x5031", connection: "USB", connected: true }];
      case "set_dpi": case "apply_button_mappings": case "set_minimize_to_tray": return;
      case "is_process_running": return false;
      case "inspect_dpi_hardware": return [{ preview: true, interfaceNumber: 2, usagePage: "0xffff", note: "Simulated read-only diagnostic" }];
      case "ask_local_assistant": return "Hi! I'm Serx. Open DPI & sensitivity to adjust your X1, or Buttons to personalise your side buttons. This is a simulated reply used to check the interface.";
      case "test_button_action": return;
      default: throw new Error("Preview command not supported: " + command);
    }
  });
}

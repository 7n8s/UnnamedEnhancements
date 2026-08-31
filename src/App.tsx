import { invoke } from "@tauri-apps/api/core";
import { ArrowDownToLine, ArrowRight, Bot, Check, ChevronRight, CircleHelp, Gauge, Gamepad2, Keyboard, Mouse, Palette, Play, Plus, RefreshCw, SendHorizontal, Settings, ShieldCheck, SlidersHorizontal, Trash2, Upload, X } from "lucide-react";
import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";

type MouseDevice = { id: string; name: string; manufacturer: string | null; vid: string | null; pid: string | null; connection: string; connected: boolean };
type Tab = "overview" | "buttons" | "performance" | "profiles" | "tester" | "help" | "assistant" | "settings";
type BackgroundMode = "default" | "solid" | "gradient" | "image";
type ImageFit = "cover" | "contain" | "stretch";
type GlassMode = "regular" | "clear";
type LocalMessage = { id: string; role: "user" | "assistant"; content: string };
type AppNotification = { id: string; title: string; detail: string };

const tabs: { id: Tab; label: string; icon: typeof Mouse; description: string }[] = [
  { id: "overview", label: "Overview", icon: Mouse, description: "Your mouse, at a glance." },
  { id: "buttons", label: "Buttons", icon: Keyboard, description: "Make your side buttons work for you." },
  { id: "performance", label: "DPI & sensitivity", icon: Gauge, description: "Find the movement that feels right." },
  { id: "profiles", label: "Profiles", icon: Gamepad2, description: "A setup for every part of your day." },
  { id: "tester", label: "Mouse tester", icon: SlidersHorizontal, description: "A simple space to check your inputs." },
  { id: "assistant", label: "Serx", icon: Bot, description: "Your optional, on-device assistant." },
  { id: "help", label: "Help & diagnostics", icon: CircleHelp, description: "A little guidance when you need it." },
  { id: "settings", label: "Appearance", icon: Settings, description: "Make this space feel like yours." },
];
const sidebarGroups: { label: string; ids: Tab[] }[] = [
  { label: "Workspace", ids: ["overview", "buttons", "performance", "profiles"] },
  { label: "Extras", ids: ["tester", "assistant", "help"] },
];
const buttonLabel = (button: string) => ({ "Button 1": "Left click", "Button 2": "Right click", "Button 3": "Wheel click", "Button 4": "Side button 1", "Button 5": "Side button 2", "Button 6": "DPI button" }[button] || button);

type ButtonAction = "Default" | "Keybind" | "Open File Explorer" | "Open Task Manager" | "Open Windows Settings" | "Open Email" | "Back" | "Forward" | "DPI Up" | "DPI Down" | "Custom program" | "Disabled";
type ButtonBinding = { action: ButtonAction; target?: string };
type Profile = { id: string; name: string; dpi: number; polling: string; buttons: Record<string, ButtonBinding> };

const buttonNames = ["Button 1", "Button 2", "Button 3", "Button 4", "Button 5"];
const actions: ButtonAction[] = ["Default", "Keybind", "Open File Explorer", "Open Task Manager", "Open Windows Settings", "Open Email", "Custom program", "Disabled"];
const keybindPresets = ["F", "G", "Space", "Tab", "Ctrl+F", "Alt+Tab", "Ctrl+Shift+S"];
const defaultButtons: Record<string, ButtonBinding> = {
  "Button 1": { action: "Default" }, "Button 2": { action: "Default" }, "Button 3": { action: "Default" },
  "Button 4": { action: "Back" }, "Button 5": { action: "Forward" },
};
const dpiPresets = [400, 800, 1600, 3200, 6400, 12800];
const mouseHotspots = [
  { button: "Button 1", label: "Left click", className: "mouse-zone left-click" },
  { button: "Button 2", label: "Right click", className: "mouse-zone right-click" },
  { button: "Button 3", label: "Wheel click", className: "mouse-zone wheel-click" },
  { button: "Button 4", label: "Side 1", className: "mouse-zone side-one" },
  { button: "Button 5", label: "Side 2", className: "mouse-zone side-two" },
  { button: "Button 6", label: "DPI cycle", className: "mouse-zone dpi-cycle" },
];
const cloneButtons = (buttons = defaultButtons) => Object.fromEntries(buttonNames.map(button => [button, { ...(buttons[button] || { action: "Default" }) }]));
const makeProfile = (name: string, source?: Profile): Profile => ({
  id: crypto.randomUUID(), name, dpi: source?.dpi ?? 800, polling: source?.polling ?? "1000 Hz", buttons: cloneButtons(source?.buttons),
});
const loadProfiles = (): Profile[] => {
  try {
    const parsed = JSON.parse(localStorage.getItem("unnamed-profiles") || "[]") as Profile[];
    if (Array.isArray(parsed) && parsed.length) return parsed.map(profile => ({ ...profile, buttons: cloneButtons(profile.buttons) }));
  } catch { /* Use safe defaults. */ }
  return [makeProfile("Default")];
};

const backgroundStore = "unnamed-appearance";
const backgroundKey = "current-background";
const openBackgroundStore = () => new Promise<IDBDatabase>((resolve, reject) => {
  const request = indexedDB.open(backgroundStore, 1);
  request.onupgradeneeded = () => request.result.createObjectStore("images");
  request.onsuccess = () => resolve(request.result);
  request.onerror = () => reject(request.error);
});
const readBackground = async () => {
  const db = await openBackgroundStore();
  return new Promise<string>((resolve, reject) => {
    const request = db.transaction("images", "readonly").objectStore("images").get(backgroundKey);
    request.onsuccess = () => resolve(typeof request.result === "string" ? request.result : "");
    request.onerror = () => reject(request.error);
  });
};
const writeBackground = async (image: string) => {
  const db = await openBackgroundStore();
  await new Promise<void>((resolve, reject) => {
    const store = db.transaction("images", "readwrite").objectStore("images");
    const request = image ? store.put(image, backgroundKey) : store.delete(backgroundKey);
    request.onsuccess = () => resolve();
    request.onerror = () => reject(request.error);
  });
};
const readAsDataUrl = (file: File) => new Promise<string>((resolve, reject) => {
  const reader = new FileReader();
  reader.onload = () => resolve(String(reader.result));
  reader.onerror = () => reject(reader.error);
  reader.readAsDataURL(file);
});
const prepareBackground = async (file: File) => {
  const original = await readAsDataUrl(file);
  // Canvas would flatten an animated GIF to a single frame, so retain it intact.
  if (file.type.toLowerCase() === "image/gif") return original;
  const image = new Image();
  await new Promise<void>((resolve, reject) => { image.onload = () => resolve(); image.onerror = () => reject(new Error("Could not read that image.")); image.src = original; });
  const longestSide = Math.max(image.naturalWidth, image.naturalHeight);
  if (longestSide >= 2560) return original;
  const scale = 2560 / longestSide;
  const canvas = document.createElement("canvas");
  canvas.width = Math.round(image.naturalWidth * scale);
  canvas.height = Math.round(image.naturalHeight * scale);
  const context = canvas.getContext("2d");
  if (!context) return original;
  context.imageSmoothingEnabled = true;
  context.imageSmoothingQuality = "high";
  context.drawImage(image, 0, 0, canvas.width, canvas.height);
  return canvas.toDataURL("image/webp", 0.94);
};

function hex(value: string, fallback: string) {
  const clean = value.trim().replace(/^#/, "");
  if (/^[0-9a-f]{6}$/i.test(clean)) return `#${clean.toUpperCase()}`;
  if (/^[0-9a-f]{3}$/i.test(clean)) return `#${clean.split("").map(c => c + c).join("").toUpperCase()}`;
  return fallback;
}
function HexColor({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  const [text, setText] = useState(value.slice(1));
  useEffect(() => setText(value.slice(1)), [value]);
  const commit = () => { const v = hex(text, value); setText(v.slice(1)); onChange(v); };
  return <div className="hex-color-control"><span>{label}</span><div className="hex-color-input"><input type="color" aria-label={`${label || "Glass tint"} picker`} value={value} onChange={e => onChange(e.target.value.toUpperCase())} /><b>#</b><input aria-label={`${label || "Glass tint"} hex colour`} value={text} maxLength={6} onChange={e => setText(e.target.value.replace(/[^0-9a-f]/gi, "").slice(0, 6))} onBlur={commit} onKeyDown={e => e.key === "Enter" && commit()} /></div></div>;
}

function KeybindInput({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  const display = value || "Press a key combination";
  return <input className="keybind-input" aria-label="Custom keybind" value={display} onFocus={event => { if (!value) event.currentTarget.value = ""; }} onChange={() => undefined} onKeyDown={event => {
    event.preventDefault();
    if (["Control", "Alt", "Shift", "Meta"].includes(event.key)) return;
    const key = event.key === " " ? "Space" : event.key.startsWith("Arrow") ? event.key.slice(5) : event.key.length === 1 ? event.key.toUpperCase() : event.key;
    if (!/^(?:[A-Z0-9]|F(?:[1-9]|1[0-9]|2[0-4])|Space|Tab|Enter|Escape|Backspace|Delete|Up|Down|Left|Right)$/.test(key)) return;
    const parts = [event.ctrlKey && "Ctrl", event.altKey && "Alt", event.shiftKey && "Shift", event.metaKey && "Win", key].filter(Boolean);
    onChange(parts.join("+"));
  }} onBlur={event => { event.currentTarget.value = display; }} />;
}

function DpiInput({ value, onApply, compact = false, minimum = 50, maximum = 40000 }: { value: number; onApply: (value: number) => void; compact?: boolean; minimum?: number; maximum?: number }) {
  const [text, setText] = useState(String(value));
  useEffect(() => setText(String(value)), [value]);
  const commit = () => {
    if (!text.trim()) { setText(String(value)); return; }
    const parsed = Number(text.replace(/[^0-9]/g, ""));
    const dpi = Number.isFinite(parsed) ? Math.min(maximum, Math.max(minimum, Math.round(parsed / 50) * 50)) : value;
    setText(String(dpi));
    if (dpi !== value) onApply(dpi);
  };
  return <div className={compact ? "dpi-input compact" : "dpi-input"}><input aria-label="DPI value" value={text} inputMode="numeric" onChange={event => setText(event.target.value.replace(/[^0-9]/g, ""))} onBlur={commit} onKeyDown={event => { if (event.key === "Enter") { event.currentTarget.blur(); } }}/><span>DPI</span></div>;
}

export default function App() {
  const [profiles, setProfiles] = useState<Profile[]>(loadProfiles);
  const [profile, setProfile] = useState(() => localStorage.getItem("unnamed-active-profile") || "");
  const [userName, setUserName] = useState(() => localStorage.getItem("unnamed-user-name") || "");
  const [newProfileName, setNewProfileName] = useState("");
  const [selectedButton, setSelectedButton] = useState("Button 1");
  const [renameTarget, setRenameTarget] = useState<Profile | null>(null);
  const [renameText, setRenameText] = useState("");
  const [deleteTarget, setDeleteTarget] = useState<Profile | null>(null);
  const [diagnosticsLoading, setDiagnosticsLoading] = useState(false);
  const [dpiStatus, setDpiStatus] = useState("Saved in this profile");
  const chatEnd = useRef<HTMLDivElement>(null);
  const [tab, setTab] = useState<Tab>("overview");
  useLayoutEffect(() => { document.getElementById("main-content")?.scrollTo({ top: 0, behavior: "instant" }); }, [tab]);
  const [mice, setMice] = useState<MouseDevice[]>([]);
  const [showOtherDevices, setShowOtherDevices] = useState(false);
  const [loading, setLoading] = useState(true);
  const [localQuestion, setLocalQuestion] = useState("");
  const [localMessages, setLocalMessages] = useState<LocalMessage[]>([]);
  const [localLoading, setLocalLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dpi, setDpi] = useState(800);
  const [dpiDiagnostics, setDpiDiagnostics] = useState<string | null>(null);
  const [textScale, setTextScale] = useState(() => Number(localStorage.getItem("unnamed-text-scale") || 100));
  const [accentColor, setAccentColor] = useState(() => localStorage.getItem("unnamed-accent-color") || "#D8B88A");
  const [layoutDensity, setLayoutDensity] = useState<"comfortable" | "compact">(() => localStorage.getItem("unnamed-layout-density") === "compact" ? "compact" : "comfortable");
  const [motionEnabled, setMotionEnabled] = useState(() => localStorage.getItem("unnamed-motion-enabled") !== "false");
  const [minimizeToTray, setMinimizeToTray] = useState(() => localStorage.getItem("unnamed-minimise-to-tray") === "true");
  const dpiTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastAppliedProfile = useRef<string | null>(null);
  const [notificationQueue, setNotificationQueue] = useState<AppNotification[]>([]);
  const [activeNotification, setActiveNotification] = useState<AppNotification | null>(null);
  const [notificationLeaving, setNotificationLeaving] = useState(false);
  const activeProfile = profiles.find(candidate => candidate.id === profile) ?? profiles[0];
  const notify = (title: string, detail: string) => setNotificationQueue(items => [...items, { id: crypto.randomUUID(), title, detail }].slice(-5));
  const buttons = activeProfile?.buttons ?? defaultButtons;

  const [bgMode, setBgMode] = useState<BackgroundMode>(() => (localStorage.getItem("unnamed-bg-mode") as BackgroundMode) || "default");
  const [bgColor, setBgColor] = useState(() => localStorage.getItem("unnamed-bg-color") || "#101112");
  const [gradA, setGradA] = useState(() => localStorage.getItem("unnamed-bg-gradient-from") || "#101112");
  const [gradB, setGradB] = useState(() => localStorage.getItem("unnamed-bg-gradient-to") || "#202124");
  const [bgImage, setBgImage] = useState("");
  const [backgroundReady, setBackgroundReady] = useState(false);
  const [fit, setFit] = useState<ImageFit>(() => (localStorage.getItem("unnamed-bg-fit") as ImageFit) || "cover");
  const [bgFocus, setBgFocus] = useState(() => localStorage.getItem("unnamed-bg-focus") || "center");
  const [bgOpacity, setBgOpacity] = useState(() => Number(localStorage.getItem("unnamed-bg-opacity") || 100));
  const [bgBlur, setBgBlur] = useState(() => Number(localStorage.getItem("unnamed-bg-blur") || 0));
  const [bgSaturation, setBgSaturation] = useState(() => Number(localStorage.getItem("unnamed-bg-saturation") || 100));
  const [uiScale, setUiScale] = useState(() => Number(localStorage.getItem("unnamed-ui-scale") || 100));
  const [testedInputs, setTestedInputs] = useState<string[]>([]);
  const [testerStatus, setTesterStatus] = useState("Waiting for an input");
  const [autoSwitchEnabled, setAutoSwitchEnabled] = useState(() => localStorage.getItem("unnamed-auto-switch") === "true");
  const [watchedProgram, setWatchedProgram] = useState(() => localStorage.getItem("unnamed-watched-program") || "");
  const [autoSwitchProfile, setAutoSwitchProfile] = useState(() => localStorage.getItem("unnamed-auto-profile") || "");
  const [autoSwitchStatus, setAutoSwitchStatus] = useState("Not watching an app");
  const profileImportRef = useRef<HTMLInputElement>(null);

  const [glassMode, setGlassMode] = useState<GlassMode>(() => (localStorage.getItem("unnamed-glass-mode") as GlassMode) || "regular");
  const [glassOpacity, setGlassOpacity] = useState(() => Number(localStorage.getItem("unnamed-glass-opacity") || 64));
  const [glassBlur, setGlassBlur] = useState(() => Number(localStorage.getItem("unnamed-glass-blur") || 22));
  const [glassTint, setGlassTint] = useState(() => localStorage.getItem("unnamed-glass-tint") || "#1E2225");
  const [glassBorder, setGlassBorder] = useState(() => Number(localStorage.getItem("unnamed-glass-border") || 42));
  const [glassRadius, setGlassRadius] = useState(() => Number(localStorage.getItem("unnamed-glass-radius") || 16));

  const mouse = mice.find(m => m.connected) ?? mice[0];
  const isG305 = /G30[45]/i.test(mouse?.name || "");
  const isModelO = mouse?.name.includes("Model O Wired") ?? false;
  const isX1 = mouse?.name.includes("Attack Shark X1") ?? false;
  const deviceButtons = isG305 || isModelO ? [...buttonNames, "Button 6"] : buttonNames;
  const deviceImage = isG305 ? "/assets/logitech/g305-top.png" : isModelO ? "/assets/glorious/model-o-wired-top.png" : "/assets/x1/attack-shark-x1-top.png";
  const sideButtonView = !isG305 && !isModelO && (selectedButton === "Button 4" || selectedButton === "Button 5");
  const buttonMapImage = sideButtonView ? "/assets/x1/attack-shark-x1-side.png" : deviceImage;
  const dpiMinimum = isG305 ? 200 : 50;
  const dpiMaximum = isG305 || isModelO ? 12000 : 40000;
  const connected = Boolean(mouse?.connected);
  const connectionLabel = mouse?.pid === "0x5032" ? "USB-C (wired)" : mouse?.pid === "0x5031" ? "2.4 GHz receiver" : mouse?.connection || "Not reported";
  const canChangeDpi = connected && isX1;
  const canRemap = selectedButton === "Button 4" || selectedButton === "Button 5";
  const currentTab = tabs.find(item => item.id === tab)!;
  const hour = new Date().getHours();
  const greeting = hour < 12 ? "Good morning" : hour < 18 ? "Good afternoon" : "Good evening";

  useEffect(() => {
    document.documentElement.style.fontSize = (16 * textScale / 100) + "px";
    return () => { document.documentElement.style.fontSize = ""; };
  }, [textScale]);
  useEffect(() => {
    localStorage.setItem("unnamed-minimise-to-tray", String(minimizeToTray));
    void invoke("set_minimize_to_tray", { enabled: minimizeToTray }).catch(() => undefined);
  }, [minimizeToTray]);
  useEffect(() => {
    if (!renameTarget && !deleteTarget) return;
    const previous = document.activeElement as HTMLElement | null;
    const trap = (event: KeyboardEvent) => {
      if (event.key === "Escape") { setRenameTarget(null); setDeleteTarget(null); }
      if (event.key !== "Tab") return;
      const nodes = Array.from(document.querySelectorAll<HTMLElement>('.modal button:not(:disabled), .modal input:not(:disabled)'));
      const first = nodes[0], last = nodes[nodes.length - 1];
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus(); }
    };
    document.addEventListener("keydown", trap);
    return () => { document.removeEventListener("keydown", trap); previous?.focus(); };
  }, [renameTarget, deleteTarget]);

  useEffect(() => {
    if (activeNotification || !notificationQueue.length) return;
    setActiveNotification(notificationQueue[0]);
    setNotificationQueue(items => items.slice(1));
    setNotificationLeaving(false);
  }, [activeNotification, notificationQueue]);
  useEffect(() => {
    if (!activeNotification) return;
    const fadeTimer = window.setTimeout(() => setNotificationLeaving(true), 3400);
    const removeTimer = window.setTimeout(() => { setActiveNotification(null); setNotificationLeaving(false); }, 3850);
    return () => { window.clearTimeout(fadeTimer); window.clearTimeout(removeTimer); };
  }, [activeNotification]);

  useEffect(() => { localStorage.setItem("unnamed-bg-mode", bgMode); localStorage.setItem("unnamed-bg-color", bgColor); localStorage.setItem("unnamed-bg-gradient-from", gradA); localStorage.setItem("unnamed-bg-gradient-to", gradB); localStorage.setItem("unnamed-bg-fit", fit); localStorage.setItem("unnamed-bg-focus", bgFocus); localStorage.setItem("unnamed-bg-opacity", String(bgOpacity)); localStorage.setItem("unnamed-bg-blur", String(bgBlur)); localStorage.setItem("unnamed-bg-saturation", String(bgSaturation)); localStorage.setItem("unnamed-ui-scale", String(uiScale)); localStorage.setItem("unnamed-glass-mode", glassMode); localStorage.setItem("unnamed-glass-opacity", String(glassOpacity)); localStorage.setItem("unnamed-glass-blur", String(glassBlur)); localStorage.setItem("unnamed-glass-tint", glassTint); localStorage.setItem("unnamed-glass-border", String(glassBorder)); localStorage.setItem("unnamed-glass-radius", String(glassRadius)); localStorage.setItem("unnamed-text-scale", String(textScale)); localStorage.setItem("unnamed-user-name", userName.trim()); localStorage.setItem("unnamed-accent-color", accentColor); localStorage.setItem("unnamed-layout-density", layoutDensity); localStorage.setItem("unnamed-motion-enabled", String(motionEnabled)); }, [bgMode,bgColor,gradA,gradB,fit,bgFocus,bgOpacity,bgBlur,bgSaturation,uiScale,textScale,glassMode,glassOpacity,glassBlur,glassTint,glassBorder,glassRadius,userName,accentColor,layoutDensity,motionEnabled]);
  useEffect(() => {
    localStorage.setItem("unnamed-auto-switch", String(autoSwitchEnabled));
    localStorage.setItem("unnamed-watched-program", watchedProgram);
    localStorage.setItem("unnamed-auto-profile", autoSwitchProfile);
    if (!autoSwitchEnabled || !watchedProgram.trim() || !autoSwitchProfile) { setAutoSwitchStatus("Not watching an app"); return; }
    let active = true;
    const check = async () => {
      try {
        const running = await invoke<boolean>("is_process_running", { processName: watchedProgram.trim() });
        if (!active) return;
        setAutoSwitchStatus(running ? `${watchedProgram} is running` : `Waiting for ${watchedProgram}`);
        if (running) setProfile(autoSwitchProfile);
      } catch (reason) {
        if (active) setAutoSwitchStatus(reason instanceof Error ? reason.message : String(reason));
      }
    };
    void check();
    const timer = window.setInterval(() => void check(), 3000);
    return () => { active = false; window.clearInterval(timer); };
  }, [autoSwitchEnabled, watchedProgram, autoSwitchProfile]);

  useEffect(() => { let active = true; void readBackground().then(image => { if (active) setBgImage(image); }).catch(() => undefined).finally(() => { if (active) setBackgroundReady(true); }); return () => { active = false; }; }, []);
  useEffect(() => { if (backgroundReady) void writeBackground(bgImage).catch(() => setError("Could not save that background image.")); }, [bgImage, backgroundReady]);
  useEffect(() => {
    if (!activeProfile) return;
    if (profile !== activeProfile.id) setProfile(activeProfile.id);
    localStorage.setItem("unnamed-profiles", JSON.stringify(profiles));
    localStorage.setItem("unnamed-active-profile", activeProfile.id);
  }, [profiles, profile, activeProfile]);
  useEffect(() => {
    if (!activeProfile) return;
    if (dpiTimer.current) clearTimeout(dpiTimer.current);
    const changed = lastAppliedProfile.current !== null && lastAppliedProfile.current !== activeProfile.id;
    lastAppliedProfile.current = activeProfile.id;
    setDpi(activeProfile.dpi);
    setDpiStatus("Saved in this profile");
    let cancelled = false;
    if (canChangeDpi) {
      void invoke("set_dpi", { dpi: activeProfile.dpi }).then(() => {
        if (cancelled) return;
        setDpiStatus("Applied to your X1");
        if (changed) notify("Profile applied", `${activeProfile.name} has been applied.`);
      }).catch(reason => { if (!cancelled) { setDpiStatus("Could not apply"); setError(String(reason)); } });
    } else if (changed) notify("Profile selected", `${activeProfile.name} is now active. Hardware DPI is unavailable for this device.`);
    return () => { cancelled = true; };
    // Apply only on device/profile changes, not each slider movement or rename.
  }, [activeProfile?.id, canChangeDpi]);

  useEffect(() => {
    if (!connected) return;
    void invoke("apply_button_mappings", { mappings: buttons }).catch(reason => setError(String(reason)));
  }, [buttons, connected]);
  useEffect(() => { chatEnd.current?.scrollIntoView({ behavior: motionEnabled ? "smooth" : "auto", block: "nearest" }); }, [localMessages, localLoading]);
  const refresh = useCallback(async () => {
    setLoading(true); setError(null);
    try { setMice(await invoke<MouseDevice[]>("detect_mice", { showHidden: showOtherDevices })); }
    catch (reason) { setMice([]); setError(String(reason)); }
    finally { setLoading(false); }
  }, [showOtherDevices]);
  useEffect(() => { void refresh(); }, [refresh]);
  useEffect(() => () => { if (dpiTimer.current) clearTimeout(dpiTimer.current); }, []);
  const applyDpi = (value: number) => {
    if (!canChangeDpi || value === dpi) return;
    const next = Math.min(dpiMaximum, Math.max(dpiMinimum, Math.round(value / 50) * 50));
    setDpi(next);
    setDpiStatus("Applying…");
    setProfiles(items => items.map(item => item.id === activeProfile?.id ? { ...item, dpi: next } : item));
    setError(null);
    if (dpiTimer.current) clearTimeout(dpiTimer.current);
    dpiTimer.current = setTimeout(async () => {
      try { await invoke("set_dpi", { dpi: next }); setDpiStatus("Applied to your X1"); notify("DPI changed", `Your mouse is now set to ${next.toLocaleString()} DPI.`); }
      catch (reason) { setDpiStatus("Could not apply"); setError(String(reason)); }
    }, 420);
  };
  const updateButton = (button: string, patch: Partial<ButtonBinding>) => setProfiles(items => items.map(item => item.id === activeProfile?.id ? { ...item, buttons: { ...item.buttons, [button]: { ...item.buttons[button], ...patch } } } : item));
  const askLocalAssistant = async () => {
    const question = localQuestion.trim();
    if (!question || localLoading) return;
    const userMessage: LocalMessage = { id: crypto.randomUUID(), role: "user", content: question };
    setError(null);
    setLocalMessages(messages => [...messages, userMessage]);
    setLocalQuestion("");
    setLocalLoading(true);
    try {
      const deviceContext = mouse ? `${mouse.name} · ${mouse.connection} · profile ${activeProfile?.name || "Default"} · ${dpi} DPI` : "No device detected";
      const reply = await invoke<string>("ask_local_assistant", { message: question, deviceContext });
      setLocalMessages(messages => [...messages, { id: crypto.randomUUID(), role: "assistant", content: reply }]);
      notify("New message from Serx", "Your local assistant has replied.");
    } catch (e) { setError(e instanceof Error ? e.message : String(e)); }
    finally { setLocalLoading(false); }
  };
  const testButtonAction = async (binding: ButtonBinding) => { setError(null); try { await invoke("test_button_action", { action: binding.action, target: binding.target || null }); } catch (e) { setError(e instanceof Error ? e.message : String(e)); } };
  const recordMouseInput = (event: React.MouseEvent<HTMLDivElement>) => {
    event.preventDefault();
    const input = ({ 0: "Button 1", 1: "Button 3", 2: "Button 2", 3: "Button 4", 4: "Button 5" } as Record<number, string>)[event.button];
    if (!input) return;
    setTestedInputs(inputs => [input, ...inputs.filter(item => item !== input)].slice(0, 6));
    setTesterStatus(`${input.replace("Button ", "M")} · ${input} detected`);
  };
  const recordScrollInput = (event: React.WheelEvent<HTMLDivElement>) => {
    event.preventDefault();
    const input = event.deltaY < 0 ? "Wheel up" : "Wheel down";
    setTestedInputs(inputs => [input, ...inputs.filter(item => item !== input)].slice(0, 6));
    setTesterStatus(`${input} detected`);
  };
  const createProfile = () => { const name = newProfileName.trim(); if (!name) return setError("Give the new profile a name."); if (profiles.some(item => item.name.toLowerCase() === name.toLowerCase())) return setError("A profile with that name already exists."); const created = makeProfile(name, activeProfile); setProfiles(items => [...items, created]); setProfile(created.id); setNewProfileName(""); setError(null); };
  const renameProfile = () => {
    const name = renameText.trim();
    if (!renameTarget || !name) return;
    if (profiles.some(other => other.id !== renameTarget.id && other.name.toLowerCase() === name.toLowerCase())) return setError("A profile with that name already exists.");
    setProfiles(items => items.map(other => other.id === renameTarget.id ? { ...other, name } : other));
    setRenameTarget(null);
  };
  const deleteProfile = (item: Profile) => {
    if (profiles.length === 1) return setError("Keep at least one profile.");
    const remaining = profiles.filter(other => other.id !== item.id);
    setProfiles(remaining);
    if (profile === item.id) setProfile(remaining[0].id);
    if (autoSwitchProfile === item.id) { setAutoSwitchEnabled(false); setAutoSwitchProfile(""); }
    setDeleteTarget(null);
  };
  const inspectDpiHardware = async () => {
    setError(null); setDiagnosticsLoading(true);
    try { setDpiDiagnostics(JSON.stringify(await invoke("inspect_dpi_hardware"), null, 2)); }
    catch (reason) { setError(String(reason)); }
    finally { setDiagnosticsLoading(false); }
  };
  const exportProfiles = () => {
    const data = JSON.stringify({ version: 1, profiles, activeProfile: activeProfile?.name || "Default" }, null, 2);
    const anchor = document.createElement("a");
    anchor.href = URL.createObjectURL(new Blob([data], { type: "application/json" }));
    anchor.download = "unnamed-profiles.json";
    anchor.click();
    window.setTimeout(() => URL.revokeObjectURL(anchor.href), 0);
  };
  const importProfiles = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]; if (!file) return;
    try {
      const backup = JSON.parse(await file.text()) as { profiles?: Profile[]; activeProfile?: string };
      if (!Array.isArray(backup.profiles) || !backup.profiles.length) throw new Error("That file has no usable profiles.");
      const restored = backup.profiles.map(item => {
        if (!item || typeof item.name !== "string" || !item.name.trim() || !Number.isFinite(item.dpi) || item.dpi < 50 || item.dpi > 40000) throw new Error("This backup contains an invalid profile. Your existing profiles were not changed.");
        const bindings = cloneButtons(item.buttons);
        for (const binding of Object.values(bindings)) {
          if (!["Default", "Keybind", "Open File Explorer", "Open Task Manager", "Open Windows Settings", "Open Email", "Back", "Forward", "DPI Up", "DPI Down", "Custom program", "Disabled"].includes(binding.action) || (binding.target !== undefined && typeof binding.target !== "string")) throw new Error("This backup contains an invalid button assignment.");
        }
        return { ...makeProfile(item.name.trim().slice(0,40)), dpi: Math.round(item.dpi / 50) * 50, polling: typeof item.polling === "string" ? item.polling : "1000 Hz", buttons: bindings };
      });
      setProfiles(restored);
      setProfile(restored.find(item => item.name === backup.activeProfile)?.id || restored[0].id);
      setError(null);
    } catch (reason) { setError(reason instanceof Error ? reason.message : "Could not import that profile backup."); }
    finally { event.target.value = ""; }
  };
  const importBackground = async (event: React.ChangeEvent<HTMLInputElement>) => { const file = event.target.files?.[0]; if (!file) return; if (!/^image\/(png|jpeg|webp|gif)$/i.test(file.type)) return setError("Use PNG, JPG/JPEG, WebP, or GIF."); if (file.size > 40 * 1024 * 1024) return setError("Background images must be 40 MB or smaller."); setError(null); try { setBgImage(await prepareBackground(file)); setBgMode("image"); } catch (reason) { setError(reason instanceof Error ? reason.message : "Could not prepare that image."); } finally { event.target.value = ""; } };
  const resetAppearance = () => { setBgMode("default"); setBgImage(""); setBgFocus("center"); setBgOpacity(100); setBgBlur(0); setBgSaturation(100); setGlassMode("regular"); setGlassOpacity(64); setGlassBlur(22); setGlassTint("#1E2225"); setGlassBorder(42); setGlassRadius(16); setUiScale(100); setTextScale(100); setAccentColor("#D8B88A"); setLayoutDensity("comfortable"); setMotionEnabled(true); };

  const background: React.CSSProperties = bgMode === "solid" ? { background: bgColor } : bgMode === "gradient" ? { background: `linear-gradient(135deg, ${gradA}, ${gradB})` } : bgMode === "image" && bgImage ? { backgroundImage: `url(${bgImage})`, backgroundSize: fit === "stretch" ? "100% 100%" : fit, backgroundPosition: bgFocus, backgroundRepeat: "no-repeat" } : { background: "radial-gradient(circle at 72% 0%, rgba(166, 132, 88, .12), transparent 34%), radial-gradient(circle at 6% 94%, rgba(79, 100, 97, .08), transparent 38%), linear-gradient(145deg, #131516 0%, #1b1c1c 50%, #111415 100%)" };

  const glassStyle = { "--glass-alpha": String(glassOpacity / 100), "--glass-blur": `${glassBlur}px`, "--glass-tint": glassTint, "--glass-border": String(glassBorder / 100), "--glass-radius": `${glassRadius}px`, "--text-scale": String(textScale / 100), "--cozy-accent": accentColor } as React.CSSProperties;
  const changeTab = (next: Tab) => {
    if (next === tab) return;
    const page = document as Document & { startViewTransition?: (update: () => void) => unknown };
    if (!motionEnabled || window.matchMedia("(prefers-reduced-motion: reduce)").matches || !page.startViewTransition) {
      setTab(next);
      return;
    }
    page.startViewTransition(() => setTab(next));
  };


  return <div className={"app-shell glass-"+glassMode+" density-"+layoutDensity+" "+(motionEnabled ? "motion-on" : "motion-off")} style={glassStyle}>
    <div className="background-layer" style={{ ...background, opacity: bgMode === "default" ? 1 : bgOpacity / 100, filter: "blur("+bgBlur+"px) saturate("+bgSaturation+"%)" }} />
    <div className="background-shade" />
    <div className="notification-stack" aria-live="polite">{activeNotification && <article className={"app-notification "+(notificationLeaving ? "leaving" : "")} key={activeNotification.id}><span className="notification-icon"><Check size={18}/></span><div><strong>{activeNotification.title}</strong><p>{activeNotification.detail}</p></div></article>}</div>
    <div className="ui-scale-layer" style={{ "--ui-scale": uiScale / 100 } as React.CSSProperties}>
      <aside className="sidebar">
        <button className="brand" onClick={() => changeTab("overview")} aria-label="Unnamed home"><span className="brand-mark"><Mouse size={23}/></span><span><strong>unnamed<span className="brand-period">.</span></strong><small>YOUR DEVICE STUDIO</small></span></button>
        <nav className="nav-list" aria-label="Channels">{sidebarGroups.map(group => <div className="nav-group" key={group.label}><span className="nav-caption">{group.label}</span>{group.ids.map(id => { const item = tabs.find(candidate => candidate.id === id)!; const Icon = item.icon; return <button className={"nav-item "+(tab === id ? "active" : "")} key={id} aria-current={tab === id ? "page" : undefined} onClick={() => changeTab(id)}><Icon size={18}/><span>{item.label}</span>{tab === id && <span className="active-mark"/>}</button>; })}</div>)}</nav>
        <div className="sidebar-bottom">
          <button className="current-device" onClick={() => changeTab("overview")} aria-label="Show current device">{connected ? <img src={deviceImage} alt=""/> : <Mouse size={30}/>}<span><small><i className={"device-dot "+(connected ? "online" : "")}/>{connected ? "CONNECTED" : "NO DEVICE"}</small><strong>{mouse?.name || "Connect your mouse"}</strong><em>{connected ? connectionLabel : "USB or wireless receiver"}</em></span></button>
          <button className={"nav-item settings-nav "+(tab === "settings" ? "active" : "")} onClick={() => changeTab("settings")} aria-current={tab === "settings" ? "page" : undefined}><Settings size={18}/><span>Appearance</span><ChevronRight size={14}/></button>
          <div className="sidebar-footer"><span>Made for your everyday.</span><span>v0.3.0</span></div>
        </div>
      </aside>
      <main className="content" id="main-content">
        <header className="topbar"><div><span className="eyebrow">{tab === "overview" ? "A SPACE THAT'S YOURS" : "YOUR WORKSPACE"}</span><h1>{tab === "overview" ? greeting+(userName.trim() ? ", "+userName.trim() : "")+"." : currentTab.label}</h1><p>{currentTab.description}</p></div><label className="active-profile"><Gamepad2 size={17}/><span><small>ACTIVE PROFILE</small><select aria-label="Active profile" value={activeProfile?.id || ""} onChange={e => setProfile(e.target.value)}>{profiles.map(item => <option key={item.id} value={item.id}>{item.name}</option>)}</select></span></label></header>
        {error && <div className="error-banner" role="alert"><CircleHelp size={18}/><p>{error}</p><button aria-label="Dismiss error" onClick={() => setError(null)}><X size={16}/></button></div>}
        <div className="channel-content" key={tab}>
        {tab === "overview" && <>
          <section className="device-hero panel">
            <div className="device-hero-copy"><span className="status-pill"><i className={"device-dot "+(connected ? "online" : "")}/>{loading ? "Looking for your device" : connected ? "Connected and ready" : "Waiting for a mouse"}</span><span className="hero-eyebrow">YOUR DAILY DRIVER</span><h2>{mouse?.name || "A good setup starts here."}</h2><p>{connected ? connectionLabel : "Connect a mouse with its USB cable or receiver, then scan to get started."}</p>
              <div className="hero-actions"><button className="primary-button" onClick={() => changeTab(connected ? "buttons" : "help")}>{connected ? "Make it yours" : "Connection help"}<ArrowRight size={16}/></button><button className="secondary-button" onClick={() => void refresh()} disabled={loading}><RefreshCw size={15} className={loading ? "spin" : ""}/>{loading ? "Scanning…" : "Scan devices"}</button></div>
              <div className="device-meta"><span>{isX1 ? "Hardware DPI supported" : connected ? "Detection & side-button shortcuts" : "No device selected"}</span>{mouse?.vid && <code>{mouse.vid} / {mouse.pid}</code>}</div>
            </div>
            <div className="hero-visual"><div className="mouse-orbit"/>{connected ? <img src={deviceImage} alt={(mouse?.name || "Mouse")+", top view"}/> : <Mouse className="empty-mouse" strokeWidth={.7}/>}<span>{connected ? "READY WHEN YOU ARE" : "YOUR NEXT SETUP"}</span></div>
          </section>
          <div className="section-heading"><h2>The essentials</h2><span>Less friction. More control.</span></div>
          <div className="overview-grid">
            <section className="panel quick-dpi"><div className="card-heading"><span className="icon-tile"><Gauge size={20}/></span><div><h3>Find your pace</h3><p>Sensitivity, made simple.</p></div><button className="icon-button" aria-label="Open DPI settings" onClick={() => changeTab("performance")}><ArrowRight size={18}/></button></div>
              <fieldset disabled={!canChangeDpi} className="dpi-field"><DpiInput value={dpi} onApply={applyDpi} minimum={dpiMinimum} maximum={dpiMaximum}/><input aria-label="Quick DPI" type="range" min={dpiMinimum} max={dpiMaximum} step="50" value={dpi} onChange={e => applyDpi(Number(e.target.value))}/><div className="range-ends"><span>Slower movement</span><span>Faster movement</span></div><div className="dpi-preset-row">{dpiPresets.slice(0,4).map(value => <button className={dpi === value ? "selected" : ""} aria-pressed={dpi === value} key={value} onClick={() => applyDpi(value)}>{value.toLocaleString()}</button>)}</div></fieldset>
              <p className="footnote">{canChangeDpi ? dpiStatus : connected ? "Hardware DPI control is currently supported on the X1 only." : "Connect an X1 to adjust hardware DPI."}</p>
            </section>
            <section className="panel overview-profile"><span className="icon-tile"><Gamepad2 size={20}/></span><span className="eyebrow">PICK UP WHERE YOU LEFT OFF</span><h3>{activeProfile?.name || "Default"}</h3><p>Your DPI and shortcuts, kept together. Create a different setup for work, play, or anything in between.</p><div className="profile-tags"><span>{dpi.toLocaleString()} DPI saved</span><span>{Object.values(buttons).filter(b => !["Default","Back","Forward"].includes(b.action)).length} custom actions</span></div><button className="text-button" onClick={() => changeTab("profiles")}>Manage profiles <ArrowRight size={16}/></button></section>
          </div>
          <div className="overview-links"><button onClick={() => changeTab("tester")}><SlidersHorizontal size={22}/><span><strong>Check your clicks</strong><small>Buttons and scroll, in one simple test.</small></span><ArrowRight size={18}/></button><button onClick={() => changeTab("settings")}><Palette size={22}/><span><strong>Set the mood</strong><small>Your colours, your glass, your background.</small></span><ArrowRight size={18}/></button></div>
        </>}
        {tab === "buttons" && <>
          <section className="panel button-workspace">
            <div className="mouse-button-map"><div className="card-heading"><div><span className="eyebrow">POINT & PERSONALISE</span><h2>{mouse?.name || "Button layout"}</h2></div><span className="subtle-badge">{sideButtonView ? "Side view" : "Top view"}</span></div>
              <div className={"mouse-map-canvas "+(isG305 ? "g305-map" : isModelO ? "model-o-map" : "x1-map")+(sideButtonView ? " side-view-map" : "")}><img className="button-map-image" key={buttonMapImage} src={buttonMapImage} alt={sideButtonView ? "Mouse side view" : "Mouse top view"}/>{mouseHotspots.filter(zone => deviceButtons.includes(zone.button) && (!sideButtonView || ["Button 4","Button 5"].includes(zone.button))).map(zone => <button className={zone.className+" "+(selectedButton === zone.button ? "selected" : "")} key={zone.button} onClick={() => setSelectedButton(zone.button)} aria-pressed={selectedButton === zone.button} aria-label={"Select "+zone.label}/>)}</div>
              <p className="map-instruction">Select a button on the mouse or in the list below.</p><div className="button-selector">{deviceButtons.map(button => <button className={selectedButton === button ? "selected" : ""} onClick={() => setSelectedButton(button)} aria-pressed={selectedButton === button} key={button}><b>{button.replace("Button ","M")}</b><span>{buttonLabel(button)}</span></button>)}</div>
            </div>
            {(() => { const binding = buttons[selectedButton] || { action: "Default" as ButtonAction }; return <div className="button-editor"><span className="eyebrow">BUTTON ASSIGNMENT</span><h2>{buttonLabel(selectedButton)}</h2><p className="editor-intro">{canRemap ? "A small shortcut. A little time back." : "This button keeps its native mouse action."}</p>
              {!canRemap ? <div className="support-note"><ShieldCheck size={24}/><h3>Native control</h3><p>Remapping this physical button isn't supported yet. Your existing mouse action is left untouched.</p><button className="secondary-button" onClick={() => setSelectedButton("Button 4")}>Customise a side button <ArrowRight size={15}/></button></div> : <><fieldset disabled={!connected}><label className="form-field">Action<select value={["Back","Forward","DPI Up","DPI Down"].includes(binding.action) ? "Default" : binding.action} onChange={e => updateButton(selectedButton, { action: e.target.value as ButtonAction, target: ["Keybind","Custom program"].includes(e.target.value) ? binding.target || "" : undefined })}>{actions.map(action => <option key={action}>{action}</option>)}</select></label>
                {binding.action === "Keybind" && <div className="editor-section"><label className="form-field">Press a key combination<KeybindInput value={binding.target || ""} onChange={target => updateButton(selectedButton, { target })}/></label><div className="keybind-presets">{keybindPresets.map(key => <button key={key} onClick={() => updateButton(selectedButton, { target: key })} className={binding.target === key ? "selected" : ""}>{key}</button>)}</div></div>}
                {binding.action === "Custom program" && <label className="form-field">Program path<input value={binding.target || ""} placeholder="e.g. notepad.exe" onChange={e => updateButton(selectedButton, { target: e.target.value })}/></label>}
                <div className="editor-section"><span className="field-caption">QUICK PICKS</span><div className="quick-actions"><button onClick={() => updateButton(selectedButton,{action:"Keybind",target:"F"})}><Keyboard size={16}/> Press F</button><button onClick={() => updateButton(selectedButton,{action:"Open File Explorer"})}>File Explorer</button><button onClick={() => updateButton(selectedButton,{action:"Open Task Manager"})}>Task Manager</button><button onClick={() => updateButton(selectedButton,{action:"Open Email"})}>Open email</button></div></div>
                {["Open File Explorer","Open Task Manager","Open Windows Settings","Open Email","Custom program"].includes(binding.action) && <button className="secondary-button" onClick={() => void testButtonAction(binding)}><Play size={14}/> Try this action</button>}
              </fieldset><div className="inline-note"><ShieldCheck size={17}/><p>{connected ? "Saved automatically. Side-button shortcuts run while Unnamed is open and may affect other connected mice." : "Connect a mouse to enable side-button shortcuts."}</p></div></>}
            </div>; })()}
          </section>
          <button className="utility-link" onClick={() => changeTab("tester")}><SlidersHorizontal size={17}/><span>Want to check an input? Open Mouse tester.</span><ArrowRight size={16}/></button>
        </>}
        {tab === "performance" && <>
          <section className="panel sensitivity-panel"><div className="card-heading"><span className="icon-tile"><Gauge size={21}/></span><div><h2>Your sensitivity</h2><p>{canChangeDpi ? "Applied directly to your Attack Shark X1." : "Hardware adjustment is available for Attack Shark X1."}</p></div><span className="subtle-badge">{canChangeDpi ? "Hardware control" : "Unavailable"}</span></div>
            <fieldset disabled={!canChangeDpi} className="dpi-field performance-field"><div className="dpi-value-row"><div><span className="field-caption">PROFILE DPI</span><DpiInput value={dpi} onApply={applyDpi} minimum={dpiMinimum} maximum={dpiMaximum}/></div><p>Higher DPI moves the pointer farther for the same hand movement. Lower DPI gives you more room for small adjustments.</p></div><div className="sensitivity-track"><input aria-label="Mouse DPI" type="range" min={dpiMinimum} max={dpiMaximum} step="50" value={dpi} onChange={e => applyDpi(Number(e.target.value))}/><div className="range-ends"><span>{dpiMinimum} DPI <small>More hand movement</small></span><span>{dpiMaximum.toLocaleString()} DPI <small>Less hand movement</small></span></div></div><span className="field-caption">START WITH A PRESET</span><div className="dpi-preset-row full">{dpiPresets.filter(value => value >= dpiMinimum && value <= dpiMaximum).map(value => <button className={dpi === value ? "selected" : ""} key={value} aria-pressed={dpi === value} onClick={() => applyDpi(value)}>{value.toLocaleString()}<small>DPI</small></button>)}</div></fieldset>
            <div className="save-status"><Check size={15}/>{canChangeDpi ? dpiStatus : connected ? "This mouse is detected, but its DPI protocol is not verified. Use its manufacturer software." : "Connect your X1 to change DPI."}</div>
          </section>
          <div className="insight-grid"><article className="panel"><span className="eyebrow">SMALL CHANGES HELP</span><h3>Start somewhere familiar.</h3><p>Try 800 or 1,600 DPI, then make small adjustments. The best value is the one that feels comfortable for your hand and desk space.</p></article><article className="panel"><span className="eyebrow">A NOTE ON SENSITIVITY</span><h3>Your game has a say, too.</h3><p>DPI works alongside your Windows and in-game sensitivity. Keep those settings steady while finding your preferred DPI.</p></article></div>
        </>}
        {tab === "profiles" && <>
          <section className="panel"><div className="card-heading"><div><h2>Your saved setups</h2><p>DPI and button assignments, saved on this PC.</p></div><span className="subtle-badge">{profiles.length} {profiles.length === 1 ? "profile" : "profiles"}</span></div><div className="profile-create"><input aria-label="New profile name" maxLength={40} value={newProfileName} placeholder="Name a new profile…" onChange={e => setNewProfileName(e.target.value)} onKeyDown={e => e.key === "Enter" && createProfile()}/><button className="primary-button" onClick={createProfile} disabled={!newProfileName.trim()}><Plus size={16}/>Create profile</button></div>
            <div className="profile-grid">{profiles.map(item => <article className={"profile-card "+(activeProfile?.id === item.id ? "selected" : "")} key={item.id}><div className="profile-card-top"><Gamepad2 size={23}/>{activeProfile?.id === item.id && <span className="subtle-badge"><Check size={12}/>Active</span>}</div><h3>{item.name}</h3><p>{item.dpi.toLocaleString()} DPI <span>·</span> {Object.values(item.buttons).filter(b => !["Default","Back","Forward"].includes(b.action)).length} custom actions</p><div className="profile-card-actions"><button className="secondary-button" onClick={() => setProfile(item.id)} disabled={activeProfile?.id === item.id}>{activeProfile?.id === item.id ? "In use" : "Apply"}</button><button className="text-button" onClick={() => { setRenameTarget(item); setRenameText(item.name); }}>Rename</button><button className="icon-button danger-button" disabled={profiles.length === 1} onClick={() => setDeleteTarget(item)} aria-label={"Delete "+item.name}><Trash2 size={15}/></button></div></article>)}</div>
          </section>
          <section className="panel profile-utilities"><details><summary><span><RefreshCw size={19}/>Automatic switching<small>Let your setup follow an app.</small></span><ChevronRight size={17}/></summary><div className="details-body"><label className="toggle-row"><span><strong>Enable auto-switching</strong><small>Applies a profile while the named process is running. It stays selected when that app closes.</small></span><input type="checkbox" checked={autoSwitchEnabled} onChange={e => setAutoSwitchEnabled(e.target.checked)}/></label><div className="form-grid"><label className="form-field">Process name<input value={watchedProgram} placeholder="e.g. game.exe" onChange={e => setWatchedProgram(e.target.value)}/></label><label className="form-field">Use profile<select value={autoSwitchProfile} onChange={e => setAutoSwitchProfile(e.target.value)}><option value="">Choose a profile</option>{profiles.map(item => <option key={item.id} value={item.id}>{item.name}</option>)}</select></label></div><p className="footnote">{autoSwitchStatus}</p></div></details>
            <details><summary><span><ArrowDownToLine size={19}/>Backup & restore<small>Take your profiles with you.</small></span><ChevronRight size={17}/></summary><div className="details-body"><p>Export a backup before moving PCs. Importing replaces your current profiles.</p><div className="tool-actions"><button className="secondary-button" onClick={exportProfiles}><ArrowDownToLine size={15}/>Export profiles</button><button className="secondary-button" onClick={() => profileImportRef.current?.click()}><Upload size={15}/>Import backup</button><input ref={profileImportRef} className="file-input" type="file" accept="application/json" onChange={importProfiles}/></div></div></details>
          </section>
        </>}
        {tab === "tester" && <section className="panel tester-panel"><div className="card-heading"><div><span className="eyebrow">INPUT CHECK</span><h2>{testerStatus}</h2></div><button className="secondary-button" onClick={() => { setTestedInputs([]); setTesterStatus("Waiting for an input"); }}><RefreshCw size={15}/>Reset test</button></div><div className="mouse-tester-pad" tabIndex={0} aria-label="Mouse input test area" onMouseDown={recordMouseInput} onWheel={recordScrollInput} onAuxClick={e => e.preventDefault()} onContextMenu={event => event.preventDefault()}><Mouse size={46} strokeWidth={1}/><h3>Click. Scroll. Check.</h3><p>Move your cursor into this area and try each button.</p><div className="mouse-tester-inputs">{["Button 1","Button 2","Button 3","Button 4","Button 5","Wheel up","Wheel down"].map(input => <div className={testedInputs.includes(input) ? "detected" : ""} key={input}><b>{input.replace("Button ","M")}</b><span>{testedInputs.includes(input) ? <><Check size={12}/>Detected</> : "Waiting"}</span></div>)}</div></div><div className="inline-note"><ShieldCheck size={17}/><p>This checks inputs received by this window. Existing shortcuts can intercept side buttons; set them to Default when testing.</p></div></section>}
        {tab === "help" && <>
          <section className="panel help-panel"><div className="card-heading"><div><h2>A few helpful answers</h2><p>The essentials, without the manual.</p></div><CircleHelp size={23}/></div>
          {[
            ["My mouse isn't showing up", "Connect the USB cable or 2.4 GHz receiver, then scan from Overview. The X1 reports USB-C when wired and 2.4 GHz when using its receiver. Appearance → Application can include generic devices in the scan."],
            ["What can Unnamed control?", "Attack Shark X1 hardware DPI is supported. Logitech G305/G304 and Glorious Model O Wired have detection and layouts. Windows side-button shortcuts work while Unnamed is open. Native main-button remapping, polling rate, RGB, and live battery reporting are not verified, so there are no controls pretending to change them."],
            ["How do I change DPI?", "Open DPI & sensitivity. Type a number and press Enter, choose a preset, or move the slider. The X1 update is sent after you stop adjusting. A confirmation appears only after the device command succeeds."],
            ["How do shortcuts work?", "Select M4 or M5 in Buttons, then choose an action. Default keeps the existing mouse action. Keyboard shortcuts go to the focused app. The Windows hook may affect side buttons on other mice too; close Unnamed to stop its shortcuts."],
            ["Where are my profiles and backgrounds?", "They stay on this PC and survive app updates. Profiles → Backup & restore exports a portable copy of your profiles. Appearance controls your saved image or GIF, colours, glass, and scale."],
            ["Serx isn't responding", "Serx is optional and needs Ollama running with qwen2.5:3b-instruct installed. The first answer can take longer while the model loads. The app's built-in help works without AI."]
          ].map(([title,body]) => <details className="help-answer" key={title}><summary>{title}<Plus size={17}/></summary><p>{body}</p></details>)}</section>
          <section className="panel"><details className="diagnostics-details"><summary><span><SlidersHorizontal size={19}/>Device diagnostics<small>Advanced, read-only device information.</small></span><ChevronRight size={17}/></summary><div className="details-body"><p>Collects HID descriptors and feature reports for troubleshooting. Interfaces without feature reports can return “Incorrect function”; this alone does not mean the mouse is disconnected.</p><button className="secondary-button" disabled={diagnosticsLoading} onClick={() => void inspectDpiHardware()}><RefreshCw size={15} className={diagnosticsLoading ? "spin" : ""}/>{diagnosticsLoading ? "Collecting…" : "Collect diagnostics"}</button>{dpiDiagnostics && <pre className="diagnostics-output">{dpiDiagnostics}</pre>}</div></details></section>
        </>}
        {tab === "assistant" && <section className="panel local-ai-panel"><div className="card-heading"><span className="assistant-mark"><Bot size={24}/></span><div><h2>A little help from Serx</h2><p>Local on your PC · Powered by Ollama</p></div><button className="icon-button" disabled={localLoading || !localMessages.length} aria-label="Clear conversation" onClick={() => setLocalMessages([])}><Trash2 size={17}/></button></div>
          <div className="local-ai-chat" role="log" aria-live="polite">{!localMessages.length && !localLoading && <div className="chat-empty"><Bot size={36} strokeWidth={1.2}/><h3>What's on your mind?</h3><p>Ask about your setup or a PC question.<br/>Serx can explain things, but can't change your settings.</p></div>}
          {localMessages.map(message => <article className={"local-ai-message "+message.role} key={message.id}><span className="local-ai-avatar">{message.role === "assistant" ? <Bot size={17}/> : (userName.trim()[0] || "Y").toUpperCase()}</span><div><strong>{message.role === "assistant" ? "Serx" : userName.trim() || "You"}</strong><p>{message.content}</p></div></article>)}
          {localLoading && <article className="local-ai-message"><span className="local-ai-avatar"><Bot size={17}/></span><div><strong>Serx</strong><p className="thinking"><i/><i/><i/><span>Thinking…</span></p></div></article>}<div ref={chatEnd}/></div>
          <div className="local-ai-composer"><textarea aria-label="Message Serx" maxLength={4000} value={localQuestion} placeholder="Message Serx…" onChange={e => setLocalQuestion(e.target.value)} onKeyDown={e => { if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) { e.preventDefault(); void askLocalAssistant(); } }}/><div><small>Enter to send · Shift + Enter for a new line</small><button className="primary-button" aria-label="Send message" disabled={localLoading || !localQuestion.trim()} onClick={() => void askLocalAssistant()}><SendHorizontal size={18}/></button></div></div>
          <details className="local-ai-setup"><summary>First time here? Set up Serx<ChevronRight size={15}/></summary><div className="details-body"><p>Install Ollama and keep it running. Download the model once with:</p><code>ollama pull qwen2.5:3b-instruct</code><p>No account or cloud API key needed. Replies are generated locally and can be mistaken.</p></div></details>
        </section>}
        {tab === "settings" && <section className="panel page-panel background-settings glass-surface"><div className="card-heading"><div><h2>Appearance</h2><p>Your colours, your space. Everything saves automatically.</p></div></div>
          <div className="appearance-section identity-section"><h3>Make it yours</h3><p className="appearance-note">Your name is saved on this PC and appears in the welcome message.</p><label className="name-setting">Your name<input value={userName} maxLength={30} placeholder="e.g. Serx" onChange={event => setUserName(event.target.value)} /></label></div><div className="appearance-section interface-preferences"><h3>Interface</h3><p className="appearance-note">Choose the little details that make Unnamed feel like yours.</p><div className="interface-preference-grid"><HexColor label="Accent colour" value={accentColor} onChange={setAccentColor}/><label>Spacing<select value={layoutDensity} onChange={event => setLayoutDensity(event.target.value as "comfortable" | "compact")}><option value="comfortable">Comfortable</option><option value="compact">Compact</option></select></label><label className="toggle-row motion-toggle"><input type="checkbox" checked={motionEnabled} onChange={event => setMotionEnabled(event.target.checked)}/><span>Gentle motion</span></label></div></div><div className="appearance-section"><h3>Background</h3><div className="background-mode-grid">{([["default","Default"],["solid","Solid colour"],["gradient","Gradient"],["image","Image"]] as [BackgroundMode,string][]).map(([mode,label]) => <button className={`background-mode glass-control ${bgMode === mode ? "selected" : ""}`} key={mode} onClick={() => setBgMode(mode)} type="button"><Palette size={17}/><span>{label}</span></button>)}</div>
          {bgMode === "solid" && <div className="background-control"><HexColor label="Colour" value={bgColor} onChange={setBgColor}/></div>}
          {bgMode === "gradient" && <div className="gradient-controls background-control"><HexColor label="Start" value={gradA} onChange={setGradA}/><HexColor label="End" value={gradB} onChange={setGradB}/></div>}
          {bgMode === "image" && <><div className="import-box glass-control"><label className="import-button"><Upload size={15}/> Import image<input className="file-input" type="file" accept="image/png,image/jpeg,image/webp,image/gif" onChange={importBackground}/></label><span>PNG, JPG/JPEG, WebP or animated GIF · max 40 MB · smaller still images are high-quality scaled to 2560px</span></div><div className="image-controls"><label>Fit<select value={fit} onChange={e => setFit(e.target.value as ImageFit)}><option value="cover">Cover</option><option value="contain">Contain</option><option value="stretch">Stretch</option></select></label><label>Focus<select value={bgFocus} onChange={e => setBgFocus(e.target.value)}><option value="center">Centre</option><option value="top">Top</option><option value="bottom">Bottom</option><option value="left">Left</option><option value="right">Right</option></select></label><label>Background opacity <b>{bgOpacity}%</b><input type="range" min="20" max="100" value={bgOpacity} onChange={e => setBgOpacity(Number(e.target.value))}/></label><label>Background blur <b>{bgBlur}px</b><input type="range" min="0" max="24" value={bgBlur} onChange={e => setBgBlur(Number(e.target.value))}/></label><label>Image saturation <b>{bgSaturation}%</b><input type="range" min="0" max="160" value={bgSaturation} onChange={e => setBgSaturation(Number(e.target.value))}/></label></div></>}
          </div>
          <div className="appearance-section"><h3>Liquid Glass</h3><p className="appearance-note">The glass layer is separate from the background, so you can reveal more or less of your image without changing its scale.</p><div className="glass-presets"><button className={glassMode === "regular" ? "selected" : ""} onClick={() => setGlassMode("regular")} type="button">Regular</button><button className={glassMode === "clear" ? "selected" : ""} onClick={() => setGlassMode("clear")} type="button">Clear</button></div><div className="glass-controls-grid"><label>UI transparency <b>{100-glassOpacity}%</b><input type="range" min="18" max="82" value={glassOpacity} onChange={e => setGlassOpacity(Number(e.target.value))}/></label><label>Glass blur <b>{glassBlur}px</b><input type="range" min="4" max="40" value={glassBlur} onChange={e => setGlassBlur(Number(e.target.value))}/></label><label>Glass tint <HexColor label="" value={glassTint} onChange={setGlassTint}/></label><label>Border strength <b>{glassBorder}%</b><input type="range" min="10" max="80" value={glassBorder} onChange={e => setGlassBorder(Number(e.target.value))}/></label><label>Corner radius <b>{glassRadius}px</b><input type="range" min="8" max="28" value={glassRadius} onChange={e => setGlassRadius(Number(e.target.value))}/></label></div></div>
          <div className="appearance-section"><h3>Interface scale</h3><div className="scale-row"><span>Layout</span><input aria-label="Interface layout scale" type="range" min="75" max="125" step="5" value={uiScale} onChange={e => setUiScale(Number(e.target.value))}/><b>{uiScale}%</b></div><div className="scale-row text-scale-control"><span>Text</span><input aria-label="Interface text scale" type="range" min="85" max="125" step="5" value={textScale} onChange={e => setTextScale(Number(e.target.value))}/><b>{textScale}%</b></div></div>
          <button className="reset-background" onClick={resetAppearance} type="button">Reset appearance</button>
          <div className="appearance-section"><h3>Application</h3><label className="toggle-row"><span><strong>Keep running in the tray</strong><small>Closing the window keeps shortcuts active. Right-click the tray icon to fully quit.</small></span><input type="checkbox" checked={minimizeToTray} onChange={e => setMinimizeToTray(e.target.checked)}/></label><label className="toggle-row"><span><strong>Show other connected mouse devices</strong><small>Include generic pointing devices in detection.</small></span><input checked={showOtherDevices} onChange={e => setShowOtherDevices(e.target.checked)} type="checkbox"/></label><p className="settings-saved-note">Appearance and profile changes save automatically on this PC.</p></div>
        </section>}
        </div>
      </main>
    </div>
    {renameTarget && <div className="modal-backdrop" onClick={() => setRenameTarget(null)}><form className="modal panel" role="dialog" aria-modal="true" aria-labelledby="rename-title" onClick={e => e.stopPropagation()} onSubmit={e => { e.preventDefault(); renameProfile(); }} onKeyDown={e => e.key === "Escape" && setRenameTarget(null)}><h2 id="rename-title">Rename profile</h2><label className="form-field">Profile name<input autoFocus maxLength={40} value={renameText} onChange={e => setRenameText(e.target.value)}/></label><div className="modal-actions"><button type="button" className="secondary-button" onClick={() => setRenameTarget(null)}>Cancel</button><button className="primary-button" disabled={!renameText.trim()}>Save name</button></div></form></div>}
    {deleteTarget && <div className="modal-backdrop" onClick={() => setDeleteTarget(null)}><section className="modal panel" role="dialog" aria-modal="true" aria-labelledby="delete-title" onClick={e => e.stopPropagation()} onKeyDown={e => e.key === "Escape" && setDeleteTarget(null)}><h2 id="delete-title">Delete {deleteTarget.name}?</h2><p>This removes its saved setup from this PC. Your other profiles will stay.</p><div className="modal-actions"><button autoFocus className="secondary-button" onClick={() => setDeleteTarget(null)}>Keep profile</button><button className="primary-button" onClick={() => deleteProfile(deleteTarget)}>Delete profile</button></div></section></div>}
  </div>;
}

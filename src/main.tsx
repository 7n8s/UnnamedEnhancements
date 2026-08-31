import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./style.css";

async function mount() {
  // The optional UI fixture never ships in production or talks to hardware.
  if (import.meta.env.DEV && new URLSearchParams(location.search).has("demo")) {
    const { setupPreview } = await import("./preview");
    setupPreview();
  }
  ReactDOM.createRoot(document.getElementById("root")!).render(<React.StrictMode><App /></React.StrictMode>);
}
void mount();

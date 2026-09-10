import { StrictMode, useEffect } from "react";
import { createRoot } from "react-dom/client";
import { Toaster } from "sonner";
import "@fontsource/cairo/400.css";
import "@fontsource/cairo/700.css";
import "@fontsource/cairo/900.css";
import "./index.css";
import App from "./App";
import { useGarageStore } from "@/stores/garageStore";

function Root() {
  const start = useGarageStore((s) => s.start);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void start().then((fn) => {
      if (disposed) fn();
      else unlisten = fn;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [start]);

  return (
    <>
      <App />
      <Toaster richColors position="top-center" />
    </>
  );
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Root />
  </StrictMode>,
);

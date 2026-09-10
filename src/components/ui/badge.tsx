import * as React from "react";
import { cn } from "@/lib/utils";

export function Badge({
  className,
  variant = "default",
  ...props
}: React.HTMLAttributes<HTMLDivElement> & { variant?: "default" | "ready" | "busy" | "muted" }) {
  const styles = {
    default: "bg-primary text-primary-foreground",
    ready: "bg-emerald-100 text-emerald-800",
    busy: "bg-red-100 text-red-800",
    muted: "bg-slate-100 text-slate-700",
  };
  return (
    <div
      className={cn(
        "inline-flex items-center rounded-full px-3 py-1 text-sm font-bold",
        styles[variant],
        className,
      )}
      {...props}
    />
  );
}

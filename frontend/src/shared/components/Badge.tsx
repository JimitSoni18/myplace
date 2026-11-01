import { JSX } from "solid-js";

export interface BadgeProps {
  variant?:
    | "default"
    | "success"
    | "warning"
    | "danger"
    | "info"
    | "residential"
    | "commercial"
    | "land";
  size?: "sm" | "md";
  children: JSX.Element;
  class?: string;
}

export function Badge(props: BadgeProps) {
  const variantClasses = {
    default: "bg-slate-800 text-slate-300 border-slate-700",
    success: "bg-emerald-950/70 text-emerald-300 border-emerald-800/60",
    warning: "bg-amber-950/70 text-amber-300 border-amber-800/60",
    danger: "bg-rose-950/70 text-rose-300 border-rose-800/60",
    info: "bg-indigo-950/70 text-indigo-300 border-indigo-800/60",
    residential: "bg-blue-950/70 text-blue-300 border-blue-800/60",
    commercial: "bg-purple-950/70 text-purple-300 border-purple-800/60",
    land: "bg-emerald-950/70 text-emerald-300 border-emerald-800/60",
  }[props.variant || "default"];

  const sizeClasses = {
    sm: "px-2 py-0.5 text-xs font-medium",
    md: "px-2.5 py-1 text-xs font-semibold",
  }[props.size || "sm"];

  return (
    <span
      class={`inline-flex items-center rounded-full border tracking-wide uppercase ${variantClasses} ${sizeClasses} ${
        props.class || ""
      }`}
    >
      {props.children}
    </span>
  );
}

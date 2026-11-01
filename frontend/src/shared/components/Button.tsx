import { JSX, splitProps } from "solid-js";

export interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "danger" | "ghost" | "outline";
  size?: "sm" | "md" | "lg";
  loading?: boolean;
}

export function Button(props: ButtonProps) {
  const [local, rest] = splitProps(props, ["variant", "size", "loading", "children", "class", "disabled"]);

  const variantClasses = {
    primary: "bg-indigo-600 hover:bg-indigo-500 text-white shadow-sm focus-visible:outline-indigo-500 active:bg-indigo-700",
    secondary: "bg-slate-800 hover:bg-slate-700 text-slate-100 border border-slate-700 focus-visible:outline-slate-600 active:bg-slate-850",
    danger: "bg-rose-600 hover:bg-rose-500 text-white shadow-sm focus-visible:outline-rose-500 active:bg-rose-700",
    ghost: "text-slate-300 hover:text-white hover:bg-slate-800/60 focus-visible:outline-slate-600",
    outline: "border border-slate-700 hover:border-slate-600 text-slate-300 hover:text-white bg-transparent",
  }[local.variant || "primary"];

  const sizeClasses = {
    sm: "px-2.5 py-1.5 text-xs font-medium rounded-md gap-1.5",
    md: "px-3.5 py-2 text-sm font-semibold rounded-lg gap-2",
    lg: "px-4 py-2.5 text-base font-semibold rounded-lg gap-2.5",
  }[local.size || "md"];

  return (
    <button
      {...rest}
      disabled={local.disabled || local.loading}
      class={`inline-flex items-center justify-center transition-all duration-150 focus-visible:outline-2 focus-visible:outline-offset-2 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer ${variantClasses} ${sizeClasses} ${local.class || ""}`}
    >
      {local.loading && (
        <svg
          class="animate-spin -ml-0.5 h-4 w-4 text-current"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
        >
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path
            class="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          ></path>
        </svg>
      )}
      {local.children}
    </button>
  );
}

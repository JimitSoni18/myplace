import { JSX, splitProps } from "solid-js";

export interface InputProps extends JSX.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  helperText?: string;
}

export function Input(props: InputProps) {
  const [local, rest] = splitProps(props, ["label", "error", "helperText", "class", "id"]);
  const inputId = local.id || (local.label ? local.label.toLowerCase().replace(/\s+/g, "-") : undefined);

  return (
    <div class="w-full">
      {local.label && (
        <label for={inputId} class="block text-xs font-medium text-slate-300 mb-1.5">
          {local.label}
          {props.required && <span class="text-rose-400 ml-0.5">*</span>}
        </label>
      )}
      <input
        {...rest}
        id={inputId}
        class={`w-full bg-slate-900 border rounded-lg px-3.5 py-2 text-sm text-slate-100 placeholder-slate-500 transition-colors focus:outline-none focus:ring-2 focus:ring-indigo-500/30 ${
          local.error
            ? "border-rose-500 focus:border-rose-500 focus:ring-rose-500/20"
            : "border-slate-800 focus:border-indigo-500 hover:border-slate-700"
        } ${local.class || ""}`}
      />
      {local.error ? (
        <p class="mt-1 text-xs text-rose-400">{local.error}</p>
      ) : local.helperText ? (
        <p class="mt-1 text-xs text-slate-500">{local.helperText}</p>
      ) : null}
    </div>
  );
}

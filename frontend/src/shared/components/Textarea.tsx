import { JSX, splitProps } from "solid-js";

export interface TextareaProps extends JSX.TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
  error?: string;
  helperText?: string;
}

export function Textarea(props: TextareaProps) {
  const [local, rest] = splitProps(props, ["label", "error", "helperText", "class", "id"]);
  const textareaId = local.id || (local.label ? local.label.toLowerCase().replace(/\s+/g, "-") : undefined);

  return (
    <div class="w-full">
      {local.label && (
        <label for={textareaId} class="block text-xs font-medium text-slate-300 mb-1.5">
          {local.label}
          {props.required && <span class="text-rose-400 ml-0.5">*</span>}
        </label>
      )}
      <textarea
        {...rest}
        id={textareaId}
        rows={props.rows || 3}
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

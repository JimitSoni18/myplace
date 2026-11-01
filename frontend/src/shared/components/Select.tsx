import { JSX, splitProps, For } from "solid-js";

export interface OptionItem {
  value: string | number;
  label: string;
}

export interface SelectProps extends JSX.SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  error?: string;
  options: OptionItem[];
  placeholder?: string;
}

export function Select(props: SelectProps) {
  const [local, rest] = splitProps(props, ["label", "error", "options", "placeholder", "class", "id"]);
  const selectId = local.id || (local.label ? local.label.toLowerCase().replace(/\s+/g, "-") : undefined);

  return (
    <div class="w-full">
      {local.label && (
        <label for={selectId} class="block text-xs font-medium text-slate-300 mb-1.5">
          {local.label}
          {props.required && <span class="text-rose-400 ml-0.5">*</span>}
        </label>
      )}
      <select
        {...rest}
        id={selectId}
        class={`w-full bg-slate-900 border rounded-lg px-3.5 py-2 text-sm text-slate-100 placeholder-slate-500 transition-colors focus:outline-none focus:ring-2 focus:ring-indigo-500/30 ${
          local.error
            ? "border-rose-500 focus:border-rose-500 focus:ring-rose-500/20"
            : "border-slate-800 focus:border-indigo-500 hover:border-slate-700"
        } ${local.class || ""}`}
      >
        {local.placeholder && (
          <option value="" disabled selected={!props.value}>
            {local.placeholder}
          </option>
        )}
        <For each={local.options}>
          {(opt) => (
            <option value={opt.value} selected={props.value === opt.value}>
              {opt.label}
            </option>
          )}
        </For>
      </select>
      {local.error && <p class="mt-1 text-xs text-rose-400">{local.error}</p>}
    </div>
  );
}

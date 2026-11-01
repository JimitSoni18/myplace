import { JSX, createEffect, onCleanup, Show } from "solid-js";
import { Portal } from "solid-js/web";

export interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  description?: string;
  children: JSX.Element;
  footer?: JSX.Element;
  maxWidth?: "sm" | "md" | "lg" | "xl" | "2xl" | "4xl";
}

export function Modal(props: ModalProps) {
  createEffect(() => {
    if (props.isOpen) {
      const handleKeyDown = (e: KeyboardEvent) => {
        if (e.key === "Escape") props.onClose();
      };
      window.addEventListener("keydown", handleKeyDown);
      document.body.style.overflow = "hidden";

      onCleanup(() => {
        window.removeEventListener("keydown", handleKeyDown);
        document.body.style.overflow = "";
      });
    }
  });

  const maxWidthClass = {
    sm: "max-w-sm",
    md: "max-w-md",
    lg: "max-w-lg",
    xl: "max-w-xl",
    "2xl": "max-w-2xl",
    "4xl": "max-w-4xl",
  }[props.maxWidth || "lg"];

  return (
    <Show when={props.isOpen}>
      <Portal>
        <div class="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6 overflow-y-auto">
          {/* Backdrop */}
          <div
            class="fixed inset-0 bg-slate-950/80 backdrop-blur-sm transition-opacity"
            onClick={props.onClose}
          />

          {/* Modal Container */}
          <div
            class={`relative w-full ${maxWidthClass} bg-slate-900 border border-slate-800 rounded-2xl shadow-2xl shadow-black/50 overflow-hidden transform transition-all z-10`}
          >
            {/* Header */}
            <div class="flex items-start justify-between px-6 py-5 border-b border-slate-800/80">
              <div>
                <h3 class="text-lg font-semibold text-slate-100">{props.title}</h3>
                {props.description && (
                  <p class="mt-1 text-xs text-slate-400">{props.description}</p>
                )}
              </div>
              <button
                type="button"
                onClick={props.onClose}
                class="rounded-lg p-1.5 text-slate-400 hover:text-white hover:bg-slate-800 transition-colors cursor-pointer"
              >
                <svg
                  class="w-5 h-5"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke-width="1.5"
                  stroke="currentColor"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* Content */}
            <div class="px-6 py-5 max-h-[calc(85vh-130px)] overflow-y-auto">
              {props.children}
            </div>

            {/* Footer */}
            {props.footer && (
              <div class="flex items-center justify-end gap-3 px-6 py-4 bg-slate-950/50 border-t border-slate-800/80">
                {props.footer}
              </div>
            )}
          </div>
        </div>
      </Portal>
    </Show>
  );
}

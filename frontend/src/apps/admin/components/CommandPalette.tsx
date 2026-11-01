import { createSignal, createEffect, onCleanup, Show, For } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { Search, Building2, Home, Users, Plus, ArrowRight, X } from "lucide-solid";
import { api } from "@/shared/api/client";
import { SearchResults } from "@/shared/api/types";

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
}

export function CommandPalette(props: CommandPaletteProps) {
  const navigate = useNavigate();
  const [query, setQuery] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [results, setResults] = createSignal<SearchResults>({
    projects: [],
    properties: [],
    owners: [],
  });

  // Debounced search
  let debounceTimer: ReturnType<typeof setTimeout>;
  createEffect(() => {
    const q = query().trim();
    clearTimeout(debounceTimer);

    if (q.length < 2) {
      setResults({ projects: [], properties: [], owners: [] });
      setLoading(false);
      return;
    }

    setLoading(true);
    debounceTimer = setTimeout(async () => {
      try {
        const res = await api.get<SearchResults>(`/api/admin/search?q=${encodeURIComponent(q)}`);
        setResults(res);
      } catch (e) {
        console.error("Search failed:", e);
      } finally {
        setLoading(false);
      }
    }, 200);
  });

  // Keyboard shortcut handler
  createEffect(() => {
    if (props.isOpen) {
      const handleKeyDown = (e: KeyboardEvent) => {
        if (e.key === "Escape") {
          props.onClose();
        }
      };
      window.addEventListener("keydown", handleKeyDown);
      onCleanup(() => window.removeEventListener("keydown", handleKeyDown));
    }
  });

  const handleSelect = (url: string) => {
    props.onClose();
    setQuery("");
    navigate(url);
  };

  const hasResults = () =>
    results().projects.length > 0 ||
    results().properties.length > 0 ||
    results().owners.length > 0;

  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 z-50 flex items-start justify-center pt-16 sm:pt-24 px-4 overflow-y-auto">
        {/* Backdrop */}
        <div
          class="fixed inset-0 bg-slate-950/80 backdrop-blur-sm transition-opacity"
          onClick={props.onClose}
        />

        {/* Dialog Box */}
        <div class="relative w-full max-w-xl bg-slate-900 border border-slate-700/80 rounded-2xl shadow-2xl shadow-black/80 overflow-hidden transform transition-all z-10">
          {/* Search Input Bar */}
          <div class="flex items-center px-4 py-3.5 border-b border-slate-800 gap-3">
            <Search class="w-5 h-5 text-indigo-400 shrink-0" />
            <input
              type="text"
              autofocus
              placeholder="Search projects, units, owners or run action..."
              value={query()}
              onInput={(e) => setQuery(e.currentTarget.value)}
              class="w-full bg-transparent text-sm text-slate-100 placeholder-slate-500 focus:outline-none"
            />
            {loading() && (
              <div class="w-4 h-4 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin shrink-0"></div>
            )}
            <button
              onClick={props.onClose}
              class="p-1 text-slate-400 hover:text-white rounded-md hover:bg-slate-800"
            >
              <X class="w-4 h-4" />
            </button>
          </div>

          {/* Results List */}
          <div class="max-h-96 overflow-y-auto p-2 divide-y divide-slate-800/60">
            {/* Quick Actions (when no query) */}
            <Show when={!query().trim()}>
              <div class="py-2">
                <div class="px-3 py-1 text-[10px] font-bold uppercase tracking-wider text-slate-500">
                  Quick Actions
                </div>
                <button
                  onClick={() => handleSelect("/projects?action=new")}
                  class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs text-slate-300 hover:bg-slate-800/80 hover:text-white group cursor-pointer"
                >
                  <span class="flex items-center gap-2.5">
                    <Plus class="w-3.5 h-3.5 text-indigo-400" /> Create New Project
                  </span>
                  <ArrowRight class="w-3.5 h-3.5 opacity-0 group-hover:opacity-100 transition-opacity text-slate-500" />
                </button>
                <button
                  onClick={() => handleSelect("/properties?action=new")}
                  class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs text-slate-300 hover:bg-slate-800/80 hover:text-white group cursor-pointer"
                >
                  <span class="flex items-center gap-2.5">
                    <Plus class="w-3.5 h-3.5 text-indigo-400" /> Add Property Unit
                  </span>
                  <ArrowRight class="w-3.5 h-3.5 opacity-0 group-hover:opacity-100 transition-opacity text-slate-500" />
                </button>
                <button
                  onClick={() => handleSelect("/owners?action=new")}
                  class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs text-slate-300 hover:bg-slate-800/80 hover:text-white group cursor-pointer"
                >
                  <span class="flex items-center gap-2.5">
                    <Plus class="w-3.5 h-3.5 text-indigo-400" /> Register Partner Owner
                  </span>
                  <ArrowRight class="w-3.5 h-3.5 opacity-0 group-hover:opacity-100 transition-opacity text-slate-500" />
                </button>
              </div>
            </Show>

            {/* Projects Results */}
            <Show when={results().projects.length > 0}>
              <div class="py-2">
                <div class="px-3 py-1 text-[10px] font-bold uppercase tracking-wider text-slate-500 flex items-center gap-1.5">
                  <Building2 class="w-3 h-3" /> Projects ({results().projects.length})
                </div>
                <For each={results().projects}>
                  {(p) => (
                    <button
                      onClick={() => handleSelect(`/projects/${p.id}`)}
                      class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs hover:bg-slate-800/80 text-left group cursor-pointer"
                    >
                      <div>
                        <span class="font-medium text-slate-100 group-hover:text-indigo-400 transition-colors">
                          {p.name}
                        </span>
                        <span class="ml-2 text-[10px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-400 font-mono">
                          {p.category}
                        </span>
                      </div>
                      <ArrowRight class="w-3.5 h-3.5 opacity-0 group-hover:opacity-100 transition-opacity text-indigo-400" />
                    </button>
                  )}
                </For>
              </div>
            </Show>

            {/* Properties Results */}
            <Show when={results().properties.length > 0}>
              <div class="py-2">
                <div class="px-3 py-1 text-[10px] font-bold uppercase tracking-wider text-slate-500 flex items-center gap-1.5">
                  <Home class="w-3 h-3" /> Property Units ({results().properties.length})
                </div>
                <For each={results().properties}>
                  {(prop) => (
                    <button
                      onClick={() => handleSelect(prop.url)}
                      class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs hover:bg-slate-800/80 text-left group cursor-pointer"
                    >
                      <div>
                        <span class="font-medium text-slate-100 group-hover:text-indigo-400 transition-colors">
                          {prop.unit_number || prop.building || `Unit #${prop.id}`}
                        </span>
                        <span class="ml-2 text-slate-400 text-[11px]">in {prop.project_name}</span>
                      </div>
                      <ArrowRight class="w-3.5 h-3.5 opacity-0 group-hover:opacity-100 transition-opacity text-indigo-400" />
                    </button>
                  )}
                </For>
              </div>
            </Show>

            {/* Owners Results */}
            <Show when={results().owners.length > 0}>
              <div class="py-2">
                <div class="px-3 py-1 text-[10px] font-bold uppercase tracking-wider text-slate-500 flex items-center gap-1.5">
                  <Users class="w-3 h-3" /> Project Owners ({results().owners.length})
                </div>
                <For each={results().owners}>
                  {(o) => (
                    <button
                      onClick={() => handleSelect(`/owners/${o.id}`)}
                      class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs hover:bg-slate-800/80 text-left group cursor-pointer"
                    >
                      <div>
                        <span class="font-medium text-slate-100 group-hover:text-indigo-400 transition-colors">
                          {o.name}
                        </span>
                        <span
                          class={`ml-2 text-[10px] px-1.5 py-0.5 rounded font-semibold ${
                            o.active ? "bg-emerald-950 text-emerald-400" : "bg-rose-950 text-rose-400"
                          }`}
                        >
                          {o.active ? "Active" : "Inactive"}
                        </span>
                      </div>
                      <ArrowRight class="w-3.5 h-3.5 opacity-0 group-hover:opacity-100 transition-opacity text-indigo-400" />
                    </button>
                  )}
                </For>
              </div>
            </Show>

            {/* Empty state when searching and no results */}
            <Show when={query().trim().length >= 2 && !loading() && !hasResults()}>
              <div class="py-8 text-center text-xs text-slate-500">
                No matching projects, units, or owners found for "{query()}"
              </div>
            </Show>
          </div>

          {/* Footer keyboard hints */}
          <div class="flex items-center justify-between px-4 py-2.5 bg-slate-950/60 border-t border-slate-800/80 text-[11px] text-slate-500">
            <span>Type to search • ESC to dismiss</span>
            <span>Global Command Center</span>
          </div>
        </div>
      </div>
    </Show>
  );
}

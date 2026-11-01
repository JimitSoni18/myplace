import { createSignal, onMount, For, Show } from "solid-js";
import { A } from "@solidjs/router";
import { Building2, MapPin, ArrowRight, Layers } from "lucide-solid";
import { Badge } from "@/shared/components/Badge";
import { Input } from "@/shared/components/Input";
import { Button } from "@/shared/components/Button";
import { api } from "@/shared/api/client";
import { OwnerProjectItem } from "@/shared/api/types";
import { debounce } from "@/shared/utils/debounce";

export function ProjectsList() {
  const [projects, setProjects] = createSignal<OwnerProjectItem[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [searchQuery, setSearchQuery] = createSignal("");
  const [debouncedSearchQuery, setDebouncedSearchQuery] = createSignal("");
  const updateDebouncedSearch = debounce((v: string) => setDebouncedSearchQuery(v), 250);
  const [categoryFilter, setCategoryFilter] = createSignal<string>("ALL");

  const fetchProjects = async () => {
    setLoading(true);
    try {
      const data = await api.get<OwnerProjectItem[]>("/api/owner/projects");
      setProjects(data);
    } catch (e: any) {
      setError(e.message || "Failed to load projects");
    } finally {
      setLoading(false);
    }
  };

  onMount(fetchProjects);

  const filteredProjects = () => {
    return projects().filter((p) => {
      const matchesSearch =
        p.name.toLowerCase().includes(debouncedSearchQuery().toLowerCase()) ||
        p.location.toLowerCase().includes(debouncedSearchQuery().toLowerCase());
      const matchesCategory =
        categoryFilter() === "ALL" ||
        p.category.toUpperCase() === categoryFilter().toUpperCase();
      return matchesSearch && matchesCategory;
    });
  };

  const categories = ["ALL", "RESIDENTIAL", "COMMERCIAL", "LAND"];

  return (
    <div class="space-y-6">
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 pb-4 border-b border-slate-800">
        <div>
          <h1 class="text-2xl font-bold tracking-tight text-white">My Projects</h1>
          <p class="text-xs text-slate-400 mt-1">
            Browse and monitor developments assigned to your developer partner account
          </p>
        </div>
      </div>

      {error() && (
        <div class="p-4 bg-rose-950/50 border border-rose-800 rounded-xl text-xs text-rose-300">
          {error()}
        </div>
      )}

      {/* Filters */}
      <div class="flex flex-col sm:flex-row items-center gap-3">
        <div class="relative w-full sm:w-80">
          <Input
            placeholder="Search projects by name or location..."
            value={searchQuery()}
            onInput={(e) => {
              setSearchQuery(e.currentTarget.value);
              updateDebouncedSearch(e.currentTarget.value);
            }}
          />
        </div>
        <div class="flex items-center gap-1.5 overflow-x-auto w-full sm:w-auto pb-1 sm:pb-0">
          <For each={categories}>
            {(cat) => (
              <button
                onClick={() => setCategoryFilter(cat)}
                class={`px-3 py-1.5 rounded-lg text-xs font-medium cursor-pointer transition-colors whitespace-nowrap ${
                  categoryFilter() === cat
                    ? "bg-emerald-600 text-white font-semibold"
                    : "bg-slate-800 text-slate-300 hover:bg-slate-700"
                }`}
              >
                {cat === "ALL" ? "All Categories" : cat.charAt(0) + cat.slice(1).toLowerCase()}
              </button>
            )}
          </For>
        </div>
      </div>

      {/* Projects Grid */}
      <Show
        when={!loading()}
        fallback={
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <For each={[1, 2, 3]}>
              {() => (
                <div class="h-64 bg-slate-900 border border-slate-800 rounded-2xl animate-pulse" />
              )}
            </For>
          </div>
        }
      >
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <For each={filteredProjects()}>
            {(proj) => (
              <div class="bg-slate-900 border border-slate-800 rounded-2xl overflow-hidden flex flex-col justify-between hover:border-slate-700 transition-all hover:shadow-xl hover:shadow-emerald-950/10 group">
                <div>
                  <div class="h-44 bg-slate-950 relative overflow-hidden flex items-center justify-center">
                    <Show
                      when={proj.hero_thumb_url}
                      fallback={
                        <div class="flex flex-col items-center justify-center text-slate-600">
                          <Building2 class="w-10 h-10 mb-1 opacity-40" />
                          <span class="text-xs">No media preview</span>
                        </div>
                      }
                    >
                      <img
                        src={proj.hero_thumb_url!}
                        alt={proj.name}
                        class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                      />
                    </Show>
                    <div class="absolute top-3 right-3">
                      <Badge variant="default">{proj.category}</Badge>
                    </div>
                  </div>

                  <div class="p-5">
                    <h3 class="text-base font-bold text-white group-hover:text-emerald-400 transition-colors">
                      {proj.name}
                    </h3>
                    <div class="flex items-center gap-1.5 text-xs text-slate-400 mt-2">
                      <MapPin class="w-3.5 h-3.5 text-slate-500 shrink-0" />
                      <span class="truncate">{proj.location}</span>
                    </div>
                  </div>
                </div>

                <div class="px-5 pb-5 pt-3 border-t border-slate-850 flex items-center justify-between">
                  <div class="flex items-center gap-1 text-xs text-slate-400">
                    <Layers class="w-3.5 h-3.5 text-slate-500" />
                    <span>
                      <strong class="text-white font-mono">{proj.properties_count}</strong> units listed
                    </span>
                  </div>
                  <A href={`/projects/${proj.id}`}>
                    <Button variant="secondary" size="sm">
                      Details <ArrowRight class="w-3.5 h-3.5 ml-1" />
                    </Button>
                  </A>
                </div>
              </div>
            )}
          </For>
        </div>

        {filteredProjects().length === 0 && (
          <div class="text-center py-16 bg-slate-900/40 rounded-2xl border border-slate-800">
            <Building2 class="w-10 h-10 text-slate-600 mx-auto mb-3" />
            <p class="text-sm text-slate-300 font-medium">No projects found</p>
            <p class="text-xs text-slate-500 mt-1">Try adjusting your search query or filters</p>
          </div>
        )}
      </Show>
    </div>
  );
}

import { createSignal, onMount, For } from "solid-js";
import { A } from "@solidjs/router";
import { Building2, Home, MapPin, ArrowRight, Layers, TrendingUp } from "lucide-solid";
import { Button } from "@/shared/components/Button";
import { Badge } from "@/shared/components/Badge";
import { api } from "@/shared/api/client";
import { OwnerDashboardResponse } from "@/shared/api/types";

export function Dashboard() {
  const [data, setData] = createSignal<OwnerDashboardResponse | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);

  const fetchDashboard = async () => {
    setLoading(true);
    try {
      const res = await api.get<OwnerDashboardResponse>("/api/owner/dashboard");
      setData(res);
    } catch (e: any) {
      setError(e.message || "Failed to load partner dashboard");
    } finally {
      setLoading(false);
    }
  };

  onMount(fetchDashboard);

  return (
    <div class="space-y-8">
      <div class="pb-2 border-b border-slate-800">
        <h1 class="text-2xl font-bold tracking-tight text-white">Partner Overview</h1>
        <p class="text-xs text-slate-400 mt-1">
          Welcome back, <span class="text-emerald-400 font-semibold">{data()?.owner_name || "Partner"}</span> • Your active developments and portfolio status
        </p>
      </div>

      {error() && (
        <div class="p-4 bg-rose-950/50 border border-rose-800 rounded-xl text-xs text-rose-300">
          {error()}
        </div>
      )}

      {/* Metrics */}
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div class="bg-slate-900 border border-slate-800 p-5 rounded-2xl">
          <div class="flex items-center justify-between text-slate-400 mb-2">
            <span class="text-xs font-medium">My Projects</span>
            <Building2 class="w-4 h-4 text-emerald-400" />
          </div>
          <div class="text-2xl font-bold text-white font-mono">
            {loading() ? "..." : data()?.stats.total_projects || 0}
          </div>
        </div>

        <div class="bg-slate-900 border border-slate-800 p-5 rounded-2xl">
          <div class="flex items-center justify-between text-slate-400 mb-2">
            <span class="text-xs font-medium">Total Units</span>
            <Home class="w-4 h-4 text-teal-400" />
          </div>
          <div class="text-2xl font-bold text-white font-mono">
            {loading() ? "..." : data()?.stats.total_properties || 0}
          </div>
        </div>

        <div class="bg-slate-900 border border-slate-800 p-5 rounded-2xl">
          <div class="flex items-center justify-between text-slate-400 mb-2">
            <span class="text-xs font-medium">Residential</span>
            <Layers class="w-4 h-4 text-blue-400" />
          </div>
          <div class="text-2xl font-bold text-white font-mono">
            {loading() ? "..." : data()?.stats.total_residential || 0}
          </div>
        </div>

        <div class="bg-slate-900 border border-slate-800 p-5 rounded-2xl">
          <div class="flex items-center justify-between text-slate-400 mb-2">
            <span class="text-xs font-medium">Commercial</span>
            <TrendingUp class="w-4 h-4 text-purple-400" />
          </div>
          <div class="text-2xl font-bold text-white font-mono">
            {loading() ? "..." : data()?.stats.total_commercial || 0}
          </div>
        </div>
      </div>

      {/* Assigned Projects */}
      <div class="space-y-4">
        <div class="flex items-center justify-between">
          <h2 class="text-base font-bold text-white">Your Development Projects</h2>
          <A href="/projects" class="text-xs font-medium text-emerald-400 hover:underline">
            View All →
          </A>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
          <For each={data()?.projects}>
            {(proj) => (
              <div class="bg-slate-900 border border-slate-800 rounded-2xl p-5 flex flex-col justify-between hover:border-slate-700 transition-colors">
                <div>
                  <div class="flex items-center justify-between gap-2 mb-2">
                    <h3 class="text-base font-bold text-white truncate">{proj.name}</h3>
                    <Badge variant="default">{proj.category}</Badge>
                  </div>

                  <div class="flex items-center gap-1.5 text-xs text-slate-400 mb-4">
                    <MapPin class="w-3.5 h-3.5 text-slate-500 shrink-0" />
                    <span class="truncate">{proj.location}</span>
                  </div>
                </div>

                <div class="flex items-center justify-between pt-4 border-t border-slate-850">
                  <span class="text-xs text-slate-400 font-mono">
                    <strong class="text-white text-sm">{proj.properties_count}</strong> units
                  </span>
                  <A href={`/projects/${proj.id}`}>
                    <Button variant="secondary" size="sm">
                      Manage <ArrowRight class="w-3.5 h-3.5 ml-1" />
                    </Button>
                  </A>
                </div>
              </div>
            )}
          </For>
          {data()?.projects.length === 0 && !loading() && (
            <div class="col-span-3 text-center py-12 text-xs text-slate-500 bg-slate-900/50 rounded-2xl border border-slate-800">
              No development projects assigned to your partner account.
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

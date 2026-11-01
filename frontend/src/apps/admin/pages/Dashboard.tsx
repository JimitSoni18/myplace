import { createSignal, onMount, Show, For } from "solid-js";
import { A } from "@solidjs/router";
import {
  Building2,
  Home,
  Users,
  Layers,
  ArrowRight,
  TrendingUp,
  MapPin,
  Plus,
} from "lucide-solid";
import { Button } from "@/shared/components/Button";
import { Badge } from "@/shared/components/Badge";
import { api } from "@/shared/api/client";
import { DashboardResponse } from "@/shared/api/types";

export function Dashboard() {
  const [data, setData] = createSignal<DashboardResponse | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);

  const fetchDashboard = async () => {
    setLoading(true);
    try {
      const res = await api.get<DashboardResponse>("/api/admin/dashboard");
      setData(res);
    } catch (e: any) {
      setError(e.message || "Failed to load dashboard metrics");
    } finally {
      setLoading(false);
    }
  };

  onMount(fetchDashboard);

  return (
    <div class="space-y-8">
      {/* Welcome Banner */}
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 pb-2 border-b border-slate-800">
        <div>
          <h1 class="text-2xl font-bold tracking-tight text-white">System Dashboard</h1>
          <p class="text-xs text-slate-400 mt-1">
            Welcome back, <span class="text-indigo-400 font-semibold">{data()?.admin_name || "Admin"}</span> • Platform overview and real-time statistics
          </p>
        </div>

        <div class="flex items-center gap-2">
          <A href="/projects">
            <Button variant="secondary" size="sm">
              View All Projects
            </Button>
          </A>
          <A href="/properties">
            <Button size="sm">
              <Plus class="w-3.5 h-3.5" /> Add New Unit
            </Button>
          </A>
        </div>
      </div>

      {error() && (
        <div class="p-4 bg-rose-950/50 border border-rose-800 rounded-xl text-xs text-rose-300">
          {error()}
        </div>
      )}

      {/* Metrics Cards Grid */}
      <div class="grid grid-cols-2 lg:grid-cols-5 gap-4">
        {/* Total Owners */}
        <div class="bg-slate-900 border border-slate-800 p-5 rounded-2xl shadow-sm">
          <div class="flex items-center justify-between text-slate-400 mb-3">
            <span class="text-xs font-medium">Project Owners</span>
            <Users class="w-4 h-4 text-indigo-400" />
          </div>
          <div class="text-2xl font-bold text-white font-mono">
            {loading() ? "..." : data()?.stats.total_owners || 0}
          </div>
          <div class="text-[11px] text-slate-500 mt-1">Registered partners</div>
        </div>

        {/* Total Projects */}
        <div class="bg-slate-900 border border-slate-800 p-5 rounded-2xl shadow-sm">
          <div class="flex items-center justify-between text-slate-400 mb-3">
            <span class="text-xs font-medium">Total Projects</span>
            <Building2 class="w-4 h-4 text-indigo-400" />
          </div>
          <div class="text-2xl font-bold text-white font-mono">
            {loading() ? "..." : data()?.stats.total_projects || 0}
          </div>
          <div class="text-[11px] text-slate-500 mt-1">Active developments</div>
        </div>

        {/* Total Properties */}
        <div class="bg-slate-900 border border-slate-800 p-5 rounded-2xl shadow-sm">
          <div class="flex items-center justify-between text-slate-400 mb-3">
            <span class="text-xs font-medium">Total Units</span>
            <Home class="w-4 h-4 text-emerald-400" />
          </div>
          <div class="text-2xl font-bold text-white font-mono">
            {loading() ? "..." : data()?.stats.total_properties || 0}
          </div>
          <div class="text-[11px] text-slate-500 mt-1">Listed inventory</div>
        </div>

        {/* Residential */}
        <div class="bg-slate-900 border border-slate-800 p-5 rounded-2xl shadow-sm">
          <div class="flex items-center justify-between text-slate-400 mb-3">
            <span class="text-xs font-medium">Residential</span>
            <Layers class="w-4 h-4 text-blue-400" />
          </div>
          <div class="text-2xl font-bold text-white font-mono">
            {loading() ? "..." : data()?.stats.total_residential || 0}
          </div>
          <div class="text-[11px] text-slate-500 mt-1">Villas, flats & duplex</div>
        </div>

        {/* Commercial */}
        <div class="bg-slate-900 border border-slate-800 p-5 rounded-2xl shadow-sm col-span-2 lg:col-span-1">
          <div class="flex items-center justify-between text-slate-400 mb-3">
            <span class="text-xs font-medium">Commercial</span>
            <TrendingUp class="w-4 h-4 text-purple-400" />
          </div>
          <div class="text-2xl font-bold text-white font-mono">
            {loading() ? "..." : data()?.stats.total_commercial || 0}
          </div>
          <div class="text-[11px] text-slate-500 mt-1">Offices & retail space</div>
        </div>
      </div>

      {/* Recent Projects Section */}
      <div class="bg-slate-900 border border-slate-800 rounded-2xl overflow-hidden shadow-sm">
        <div class="flex items-center justify-between px-6 py-4 border-b border-slate-800">
          <div>
            <h2 class="text-sm font-semibold text-white">Recent Developments</h2>
            <p class="text-xs text-slate-400">Newly added projects and real-time inventory</p>
          </div>
          <A
            href="/projects"
            class="text-xs font-medium text-indigo-400 hover:text-indigo-300 flex items-center gap-1"
          >
            All projects <ArrowRight class="w-3.5 h-3.5" />
          </A>
        </div>

        <div class="overflow-x-auto">
          <table class="w-full text-left text-xs">
            <thead class="bg-slate-950/60 text-slate-400 uppercase tracking-wider font-semibold border-b border-slate-800/80">
              <tr>
                <th class="px-6 py-3">Project</th>
                <th class="px-6 py-3">Owner / Partner</th>
                <th class="px-6 py-3">Location</th>
                <th class="px-6 py-3">Category</th>
                <th class="px-6 py-3 text-right">Inventory</th>
                <th class="px-6 py-3 text-right">Action</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-800/60">
              <Show
                when={!loading() && data()?.recent_projects.length}
                fallback={
                  <tr>
                    <td colspan="6" class="px-6 py-12 text-center text-slate-500">
                      {loading() ? "Loading projects..." : "No projects registered yet."}
                    </td>
                  </tr>
                }
              >
                <For each={data()?.recent_projects}>
                  {(proj) => (
                    <tr class="hover:bg-slate-800/40 transition-colors">
                      <td class="px-6 py-4 font-semibold text-slate-100">
                        <A href={`/projects/${proj.id}`} class="hover:text-indigo-400 transition-colors">
                          {proj.name}
                        </A>
                      </td>
                      <td class="px-6 py-4 text-slate-300">{proj.owner_name}</td>
                      <td class="px-6 py-4 text-slate-400">
                        <span class="inline-flex items-center gap-1">
                          <MapPin class="w-3 h-3 text-slate-500" />
                          {proj.location}
                        </span>
                      </td>
                      <td class="px-6 py-4">
                        <Badge
                          variant={
                            proj.category === "RESIDENTIAL"
                              ? "residential"
                              : proj.category === "COMMERCIAL"
                              ? "commercial"
                              : "land"
                          }
                        >
                          {proj.category}
                        </Badge>
                      </td>
                      <td class="px-6 py-4 text-right font-mono text-slate-300">
                        <span class="text-emerald-400 font-semibold">{proj.available_properties}</span> /{" "}
                        {proj.total_properties}
                      </td>
                      <td class="px-6 py-4 text-right">
                        <A
                          href={`/projects/${proj.id}`}
                          class="inline-flex items-center gap-1 text-indigo-400 hover:text-indigo-300 font-medium"
                        >
                          Manage <ArrowRight class="w-3 h-3" />
                        </A>
                      </td>
                    </tr>
                  )}
                </For>
              </Show>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

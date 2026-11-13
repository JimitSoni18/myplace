import { createSignal, onMount, For, Show } from "solid-js";
import { useParams, A } from "@solidjs/router";
import {
  ArrowLeft,
  FileText,
  Image as ImageIcon,
  MapPin,
  Layers,
  Download,
  ExternalLink,
  ShieldCheck,
} from "lucide-solid";
import { Badge } from "@/shared/components/Badge";
import { Input } from "@/shared/components/Input";
import { api } from "@/shared/api/client";
import { OwnerProjectDetailResponse } from "@/shared/api/types";

export function ProjectDetail() {
  const params = useParams<{ id: string }>();
  const [project, setProject] = createSignal<OwnerProjectDetailResponse | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [activeTab, setActiveTab] = createSignal<"overview" | "configurations" | "media" | "documents">("overview");
  const [configSearch, setConfigSearch] = createSignal("");

  const fetchDetails = async () => {
    setLoading(true);
    try {
      const projData = await api.get<OwnerProjectDetailResponse>(`/api/owner/projects/${params.id}`);
      setProject(projData);
    } catch (e: any) {
      setError(e.message || "Failed to load project details");
    } finally {
      setLoading(false);
    }
  };

  onMount(fetchDetails);

  const filteredConfigurations = () => {
    const list = project()?.configurations || [];
    const q = configSearch().trim().toLowerCase();
    if (!q) return list;
    return list.filter((c) => {
      const name = c.name.toLowerCase();
      const bhk = c.bhk != null ? `${c.bhk} bhk` : "";
      return name.includes(q) || bhk.includes(q);
    });
  };

  const formatPrice = (price?: number | null, priceOnRequest?: boolean) => {
    if (priceOnRequest || !price || price === 0) return "Price on Request";
    return new Intl.NumberFormat("en-IN", {
      style: "currency",
      currency: "INR",
      maximumFractionDigits: 0,
    }).format(price);
  };

  const formatFileSize = (bytes?: number | null) => {
    if (!bytes) return "Unknown size";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div class="space-y-6">
      {/* Back button */}
      <div>
        <A
          href="/projects"
          class="inline-flex items-center gap-1.5 text-xs font-medium text-slate-400 hover:text-white transition-colors"
        >
          <ArrowLeft class="w-3.5 h-3.5" /> Back to Projects
        </A>
      </div>

      {error() && (
        <div class="p-4 bg-rose-950/50 border border-rose-800 rounded-xl text-xs text-rose-300">
          {error()}
        </div>
      )}

      <Show
        when={!loading() && project()}
        fallback={
          <div class="space-y-4 animate-pulse">
            <div class="h-10 bg-slate-800 rounded-xl w-1/3" />
            <div class="h-64 bg-slate-900 rounded-2xl" />
          </div>
        }
      >
        {/* Header summary */}
        <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6">
          <div class="flex flex-col md:flex-row md:items-start md:justify-between gap-4">
            <div>
              <div class="flex flex-wrap items-center gap-2.5 mb-2">
                <h1 class="text-2xl font-bold tracking-tight text-white">{project()!.name}</h1>
                <Badge variant="default">{project()!.project_type_name}</Badge>
                {project()!.rera_number && (
                  <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[11px] font-medium bg-emerald-950/70 text-emerald-400 border border-emerald-800/60">
                    <ShieldCheck class="w-3 h-3" />
                    RERA: {project()!.rera_number}
                  </span>
                )}
              </div>
              <div class="flex items-center gap-2 text-xs text-slate-400">
                <MapPin class="w-3.5 h-3.5 text-slate-500 shrink-0" />
                <span>
                  {project()!.formatted_address}
                  {project()!.area && ` • ${project()!.area}`}
                  {project()!.city && `, ${project()!.city}`}
                  {project()!.state_or_province && `, ${project()!.state_or_province}`}
                  {project()!.postal_code && ` - ${project()!.postal_code}`}
                </span>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="px-4 py-2 rounded-xl bg-slate-950 border border-slate-800 text-right">
                <span class="text-[10px] uppercase tracking-wider text-slate-400 block font-semibold">
                  Typologies
                </span>
                <span class="text-lg font-bold text-white font-mono">
                  {project()!.configurations_count}
                </span>
              </div>
              {project()!.total_land_area != null && (
                <div class="px-4 py-2 rounded-xl bg-slate-950 border border-slate-800 text-right">
                  <span class="text-[10px] uppercase tracking-wider text-slate-400 block font-semibold">
                    Land Area
                  </span>
                  <span class="text-lg font-bold text-emerald-400 font-mono">
                    {project()!.total_land_area} <span class="text-xs text-slate-400">sq.ft</span>
                  </span>
                </div>
              )}
            </div>
          </div>

          {/* Navigation Tabs */}
          <div class="flex items-center gap-2 border-b border-slate-800 mt-6 -mb-6 pb-0 overflow-x-auto">
            <button
              onClick={() => setActiveTab("overview")}
              class={`px-4 py-3 text-xs font-medium border-b-2 transition-all cursor-pointer whitespace-nowrap ${
                activeTab() === "overview"
                  ? "border-emerald-500 text-emerald-400 font-semibold"
                  : "border-transparent text-slate-400 hover:text-slate-200"
              }`}
            >
              Overview
            </button>
            <button
              onClick={() => setActiveTab("configurations")}
              class={`px-4 py-3 text-xs font-medium border-b-2 transition-all cursor-pointer whitespace-nowrap flex items-center gap-1.5 ${
                activeTab() === "configurations"
                  ? "border-emerald-500 text-emerald-400 font-semibold"
                  : "border-transparent text-slate-400 hover:text-slate-200"
              }`}
            >
              <Layers class="w-3.5 h-3.5" />
              Typologies ({project()!.configurations.length})
            </button>
            <button
              onClick={() => setActiveTab("media")}
              class={`px-4 py-3 text-xs font-medium border-b-2 transition-all cursor-pointer whitespace-nowrap flex items-center gap-1.5 ${
                activeTab() === "media"
                  ? "border-emerald-500 text-emerald-400 font-semibold"
                  : "border-transparent text-slate-400 hover:text-slate-200"
              }`}
            >
              <ImageIcon class="w-3.5 h-3.5" />
              Media Gallery ({project()!.media.length})
            </button>
            <button
              onClick={() => setActiveTab("documents")}
              class={`px-4 py-3 text-xs font-medium border-b-2 transition-all cursor-pointer whitespace-nowrap flex items-center gap-1.5 ${
                activeTab() === "documents"
                  ? "border-emerald-500 text-emerald-400 font-semibold"
                  : "border-transparent text-slate-400 hover:text-slate-200"
              }`}
            >
              <FileText class="w-3.5 h-3.5" />
              Documents ({project()!.documents.length})
            </button>
          </div>
        </div>

        {/* Tab 1: Overview */}
        <Show when={activeTab() === "overview"}>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div class="md:col-span-2 space-y-6">
              <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6">
                <h2 class="text-sm font-bold text-white mb-3">Project Description</h2>
                <div class="text-xs text-slate-300 leading-relaxed whitespace-pre-line">
                  {project()!.description || "No project description has been specified."}
                </div>
              </div>

              {/* Project Scale Metrics */}
              <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6">
                <h2 class="text-sm font-bold text-white mb-4">Scale & Specifications</h2>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
                  <div class="p-3.5 rounded-xl bg-slate-950 border border-slate-800">
                    <span class="text-[10px] text-slate-400 uppercase font-semibold block">Total Land</span>
                    <span class="text-sm font-semibold text-white font-mono mt-1 block">
                      {project()!.total_land_area != null ? `${project()!.total_land_area} sq.ft` : "—"}
                    </span>
                  </div>
                  <div class="p-3.5 rounded-xl bg-slate-950 border border-slate-800">
                    <span class="text-[10px] text-slate-400 uppercase font-semibold block">Towers</span>
                    <span class="text-sm font-semibold text-white font-mono mt-1 block">
                      {project()!.towers_count != null ? project()!.towers_count : "—"}
                    </span>
                  </div>
                  <div class="p-3.5 rounded-xl bg-slate-950 border border-slate-800">
                    <span class="text-[10px] text-slate-400 uppercase font-semibold block">Floors</span>
                    <span class="text-sm font-semibold text-white font-mono mt-1 block">
                      {project()!.floors_count != null ? project()!.floors_count : "—"}
                    </span>
                  </div>
                  <div class="p-3.5 rounded-xl bg-slate-950 border border-slate-800">
                    <span class="text-[10px] text-slate-400 uppercase font-semibold block">Planned Units</span>
                    <span class="text-sm font-semibold text-white font-mono mt-1 block">
                      {project()!.units_count != null ? project()!.units_count : "—"}
                    </span>
                  </div>
                </div>
              </div>

              {/* Milestones / Dates */}
              <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6">
                <h2 class="text-sm font-bold text-white mb-4">Project Timeline</h2>
                <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                  <div class="p-3.5 rounded-xl bg-slate-950 border border-slate-800">
                    <span class="text-[10px] text-slate-400 uppercase font-semibold block">Start Date</span>
                    <span class="text-xs font-semibold text-slate-200 mt-1 block">
                      {project()!.start_date || "Not set"}
                    </span>
                  </div>
                  <div class="p-3.5 rounded-xl bg-slate-950 border border-slate-800">
                    <span class="text-[10px] text-slate-400 uppercase font-semibold block">Launch Date</span>
                    <span class="text-xs font-semibold text-slate-200 mt-1 block">
                      {project()!.launch_date || "Not set"}
                    </span>
                  </div>
                  <div class="p-3.5 rounded-xl bg-slate-950 border border-slate-800">
                    <span class="text-[10px] text-slate-400 uppercase font-semibold block">Possession Date</span>
                    <span class="text-xs font-semibold text-emerald-400 mt-1 block">
                      {project()!.possession_date || "Not set"}
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <div class="space-y-6">
              {/* Regulatory Information */}
              <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6">
                <h2 class="text-sm font-bold text-white mb-3">Regulatory Compliance</h2>
                <div class="space-y-3 text-xs">
                  <div>
                    <span class="text-slate-500 block text-[10px] uppercase font-semibold">RERA Number</span>
                    <span class="font-mono text-slate-200">{project()!.rera_number || "Not specified"}</span>
                  </div>
                  <div>
                    <span class="text-slate-500 block text-[10px] uppercase font-semibold">RERA Authority</span>
                    <span class="text-slate-200">{project()!.rera_authority || "Not specified"}</span>
                  </div>
                </div>
              </div>

              <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6">
                <h2 class="text-sm font-bold text-white mb-3">Developer Guarantee</h2>
                <div class="p-3.5 rounded-xl bg-emerald-950/30 border border-emerald-900/50 flex items-start gap-3">
                  <ShieldCheck class="w-5 h-5 text-emerald-400 shrink-0 mt-0.5" />
                  <p class="text-xs text-emerald-300 leading-relaxed">
                    This project is registered under your authenticated developer profile. All typologies, media, and documents are synchronized with the central catalog.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </Show>

        {/* Tab 2: Typologies / Configurations */}
        <Show when={activeTab() === "configurations"}>
          <div class="space-y-4">
            <div class="flex items-center justify-between gap-4">
              <div class="relative w-full sm:w-80">
                <Input
                  placeholder="Filter typologies by name or BHK..."
                  value={configSearch()}
                  onInput={(e) => setConfigSearch(e.currentTarget.value)}
                />
              </div>
              <span class="text-xs text-slate-400 font-mono">
                Showing {filteredConfigurations().length} of {project()!.configurations.length} typologies
              </span>
            </div>

            <div class="bg-slate-900 border border-slate-800 rounded-2xl overflow-hidden">
              <div class="overflow-x-auto">
                <table class="w-full text-left border-collapse text-xs">
                  <thead>
                    <tr class="border-b border-slate-800 bg-slate-950/50 text-slate-400 uppercase font-semibold text-[10px] tracking-wider">
                      <th class="py-3 px-4">Floor Plan</th>
                      <th class="py-3 px-4">Typology</th>
                      <th class="py-3 px-4">BHK</th>
                      <th class="py-3 px-4">Carpet Area</th>
                      <th class="py-3 px-4">Built-up Area</th>
                      <th class="py-3 px-4">Super Built-up Area</th>
                      <th class="py-3 px-4">Price</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-slate-850">
                    <For each={filteredConfigurations()}>
                      {(c) => (
                        <tr class="hover:bg-slate-800/40 transition-colors">
                          <td class="py-3 px-4">
                            <Show
                              when={c.floor_plan_url}
                              fallback={
                                <div class="w-12 h-12 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-center text-slate-600 text-[10px]">
                                  No plan
                                </div>
                              }
                            >
                              <a
                                href={c.floor_plan_url!}
                                target="_blank"
                                rel="noreferrer"
                                class="block w-12 h-12 rounded-lg bg-slate-950 border border-slate-800 overflow-hidden hover:border-emerald-500 transition-colors group"
                                title="Click to view floor plan"
                              >
                                <img
                                  src={c.floor_plan_thumb_url || c.floor_plan_url!}
                                  alt={c.name}
                                  class="w-full h-full object-cover group-hover:scale-110 transition-transform"
                                />
                              </a>
                            </Show>
                          </td>
                          <td class="py-3 px-4 font-semibold text-white">
                            {c.name}
                          </td>
                          <td class="py-3 px-4 text-slate-300 font-mono">
                            {c.bhk != null ? `${c.bhk} BHK` : "—"}
                          </td>
                          <td class="py-3 px-4 font-mono text-slate-300">
                            {c.carpet_area != null ? `${c.carpet_area} sq.ft` : "—"}
                          </td>
                          <td class="py-3 px-4 font-mono text-slate-300">
                            {c.built_up_area != null ? `${c.built_up_area} sq.ft` : "—"}
                          </td>
                          <td class="py-3 px-4 font-mono text-slate-300">
                            {c.super_built_up_area != null ? `${c.super_built_up_area} sq.ft` : "—"}
                          </td>
                          <td class="py-3 px-4 font-mono font-medium text-emerald-400">
                            {formatPrice(c.price, c.price_on_request)}
                          </td>
                        </tr>
                      )}
                    </For>
                  </tbody>
                </table>
              </div>

              {filteredConfigurations().length === 0 && (
                <div class="p-8 text-center text-xs text-slate-500">
                  No typologies match your search criteria.
                </div>
              )}
            </div>
          </div>
        </Show>

        {/* Tab 3: Media Gallery */}
        <Show when={activeTab() === "media"}>
          <div class="space-y-4">
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
              <For each={project()!.media}>
                {(item) => (
                  <div class="group relative rounded-xl overflow-hidden bg-slate-950 border border-slate-800 aspect-video">
                    <img
                      src={item.thumbnail_url || item.url}
                      alt="Project asset"
                      class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                    />
                    <div class="absolute inset-0 bg-slate-950/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2">
                      <a
                        href={item.url}
                        target="_blank"
                        rel="noreferrer"
                        class="p-2 rounded-lg bg-slate-900 text-white hover:bg-slate-800 transition-colors"
                        title="Open full size"
                      >
                        <ExternalLink class="w-4 h-4" />
                      </a>
                    </div>
                  </div>
                )}
              </For>
            </div>

            {project()!.media.length === 0 && (
              <div class="p-12 text-center text-xs text-slate-500 bg-slate-900 border border-slate-800 rounded-2xl">
                No visual media uploaded for this development project.
              </div>
            )}
          </div>
        </Show>

        {/* Tab 4: Documents */}
        <Show when={activeTab() === "documents"}>
          <div class="space-y-3">
            <For each={project()!.documents}>
              {(doc) => (
                <div class="flex items-center justify-between p-4 rounded-xl bg-slate-900 border border-slate-800 hover:border-slate-700 transition-colors">
                  <div class="flex items-center gap-3">
                    <div class="p-2.5 rounded-lg bg-emerald-950/40 text-emerald-400 border border-emerald-900/50">
                      <FileText class="w-5 h-5" />
                    </div>
                    <div>
                      <h4 class="text-xs font-semibold text-white">{doc.file_name}</h4>
                      <span class="text-[10px] text-slate-500">{formatFileSize(doc.file_size)}</span>
                    </div>
                  </div>

                  <a
                    href={doc.url}
                    target="_blank"
                    rel="noreferrer"
                    download={doc.file_name}
                    class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-xs font-medium text-slate-200 transition-colors"
                  >
                    <Download class="w-3.5 h-3.5" /> Download
                  </a>
                </div>
              )}
            </For>

            {project()!.documents.length === 0 && (
              <div class="p-12 text-center text-xs text-slate-500 bg-slate-900 border border-slate-800 rounded-2xl">
                No official documents or brochures uploaded for this development.
              </div>
            )}
          </div>
        </Show>
      </Show>
    </div>
  );
}

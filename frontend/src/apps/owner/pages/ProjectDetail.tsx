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
import { OwnerProjectDetailResponse, OwnerPropertyItem } from "@/shared/api/types";

export function ProjectDetail() {
  const params = useParams<{ id: string }>();
  const [project, setProject] = createSignal<OwnerProjectDetailResponse | null>(null);
  const [properties, setProperties] = createSignal<OwnerPropertyItem[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [activeTab, setActiveTab] = createSignal<"overview" | "units" | "media" | "documents">("overview");
  const [unitSearch, setUnitSearch] = createSignal("");

  const fetchDetails = async () => {
    setLoading(true);
    try {
      const [projData, propsData] = await Promise.all([
        api.get<OwnerProjectDetailResponse>(`/api/owner/projects/${params.id}`),
        api.get<OwnerPropertyItem[]>(`/api/owner/projects/${params.id}/properties`),
      ]);
      setProject(projData);
      setProperties(propsData);
    } catch (e: any) {
      setError(e.message || "Failed to load project details");
    } finally {
      setLoading(false);
    }
  };

  onMount(fetchDetails);

  const filteredProperties = () => {
    return properties().filter((p) => {
      const q = unitSearch().toLowerCase();
      const unit = (p.unit_number || "").toLowerCase();
      const bldg = (p.building || "").toLowerCase();
      const type = p.type_name.toLowerCase();
      return unit.includes(q) || bldg.includes(q) || type.includes(q);
    });
  };

  const formatPrice = (val: number) => {
    if (!val || val === 0) return "Price on Request";
    return new Intl.NumberFormat("en-IN", {
      style: "currency",
      currency: "INR",
      maximumFractionDigits: 0,
    }).format(val);
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
              <div class="flex items-center gap-2.5 mb-2">
                <h1 class="text-2xl font-bold tracking-tight text-white">{project()!.name}</h1>
                <Badge variant="default">{project()!.category}</Badge>
              </div>
              <div class="flex items-center gap-2 text-xs text-slate-400">
                <MapPin class="w-3.5 h-3.5 text-slate-500" />
                <span>{project()!.location}</span>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="px-4 py-2 rounded-xl bg-slate-950 border border-slate-800 text-right">
                <span class="text-[10px] uppercase tracking-wider text-slate-400 block font-semibold">
                  Total Units
                </span>
                <span class="text-lg font-bold text-white font-mono">
                  {project()!.properties_count}
                </span>
              </div>
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
              onClick={() => setActiveTab("units")}
              class={`px-4 py-3 text-xs font-medium border-b-2 transition-all cursor-pointer whitespace-nowrap flex items-center gap-1.5 ${
                activeTab() === "units"
                  ? "border-emerald-500 text-emerald-400 font-semibold"
                  : "border-transparent text-slate-400 hover:text-slate-200"
              }`}
            >
              <Layers class="w-3.5 h-3.5" />
              Units ({properties().length})
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
              <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6">
                <h2 class="text-sm font-bold text-white mb-3">Developer Guarantee</h2>
                <div class="p-3.5 rounded-xl bg-emerald-950/30 border border-emerald-900/50 flex items-start gap-3">
                  <ShieldCheck class="w-5 h-5 text-emerald-400 shrink-0 mt-0.5" />
                  <p class="text-xs text-emerald-300 leading-relaxed">
                    This project is registered under your authenticated developer profile. All unit inventory, media, and documents are synchronized with the central catalog.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </Show>

        {/* Tab 2: Units Inventory */}
        <Show when={activeTab() === "units"}>
          <div class="space-y-4">
            <div class="flex items-center justify-between gap-4">
              <div class="relative w-full sm:w-80">
                <Input
                  placeholder="Filter units by number, building, or type..."
                  value={unitSearch()}
                  onInput={(e) => setUnitSearch(e.currentTarget.value)}
                />
              </div>
              <span class="text-xs text-slate-400 font-mono">
                Showing {filteredProperties().length} of {properties().length} units
              </span>
            </div>

            <div class="bg-slate-900 border border-slate-800 rounded-2xl overflow-hidden">
              <div class="overflow-x-auto">
                <table class="w-full text-left border-collapse text-xs">
                  <thead>
                    <tr class="border-b border-slate-800 bg-slate-950/50 text-slate-400 uppercase font-semibold text-[10px] tracking-wider">
                      <th class="py-3 px-4">Unit</th>
                      <th class="py-3 px-4">Building</th>
                      <th class="py-3 px-4">Type</th>
                      <th class="py-3 px-4">Area (sq.ft)</th>
                      <th class="py-3 px-4">Price</th>
                      <th class="py-3 px-4">Status</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-slate-850">
                    <For each={filteredProperties()}>
                      {(u) => (
                        <tr class="hover:bg-slate-800/40 transition-colors">
                          <td class="py-3 px-4 font-semibold text-white">
                            {u.unit_number || `Unit #${u.id}`}
                          </td>
                          <td class="py-3 px-4 text-slate-300">{u.building || "—"}</td>
                          <td class="py-3 px-4 text-slate-300">
                            <span class="px-2 py-0.5 rounded bg-slate-800 text-slate-300 text-[11px]">
                              {u.type_name}
                            </span>
                          </td>
                          <td class="py-3 px-4 font-mono text-slate-300">
                            {u.built_up_area ? `${u.built_up_area} sq.ft` : "—"}
                          </td>
                          <td class="py-3 px-4 font-mono font-medium text-emerald-400">
                            {formatPrice(u.price)}
                          </td>
                          <td class="py-3 px-4">
                            <Badge variant={u.status === "available" ? "success" : "default"}>
                              {u.status}
                            </Badge>
                          </td>
                        </tr>
                      )}
                    </For>
                  </tbody>
                </table>
              </div>

              {filteredProperties().length === 0 && (
                <div class="p-8 text-center text-xs text-slate-500">
                  No unit listings match your search criteria.
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

import { createSignal, onMount, For, Show } from "solid-js";
import { A, useSearchParams } from "@solidjs/router";
import { Building2, Plus, MapPin, ExternalLink } from "lucide-solid";
import { Button } from "@/shared/components/Button";
import { Input } from "@/shared/components/Input";
import { Select } from "@/shared/components/Select";
import { Textarea } from "@/shared/components/Textarea";
import { Badge } from "@/shared/components/Badge";
import { Modal } from "@/shared/components/Modal";
import { api } from "@/shared/api/client";
import { ProjectListItem, MetaOptionsResponse } from "@/shared/api/types";
import { debounce } from "@/shared/utils/debounce";

export function ProjectsList() {
  const [params] = useSearchParams();
  const [projects, setProjects] = createSignal<ProjectListItem[]>([]);
  const [meta, setMeta] = createSignal<MetaOptionsResponse | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [search, setSearch] = createSignal("");
  const [selectedCategory, setSelectedCategory] = createSignal("");
  const [selectedOwner, setSelectedOwner] = createSignal("");

  const debouncedFetchProjects = debounce(() => {
    fetchProjects();
  }, 300);

  // Create Project Modal state
  const [modalOpen, setModalOpen] = createSignal(params.action === "new");
  const [submitting, setSubmitting] = createSignal(false);
  const [formError, setFormError] = createSignal<string | null>(null);

  const [formName, setFormName] = createSignal("");
  const [formOwnerId, setFormOwnerId] = createSignal<number | "">("");
  const [formLocationId, setFormLocationId] = createSignal<number | "">("");
  const [formCategory, setFormCategory] = createSignal<"RESIDENTIAL" | "COMMERCIAL" | "LAND">("RESIDENTIAL");
  const [formDesc, setFormDesc] = createSignal("");
  const [formLaunchDate, setFormLaunchDate] = createSignal("");
  const [formPossessionDate, setFormPossessionDate] = createSignal("");
  const [formAmenityIds, setFormAmenityIds] = createSignal<number[]>([]);

  const fetchProjects = async () => {
    setLoading(true);
    try {
      const q = new URLSearchParams();
      if (search().trim()) q.set("q", search().trim());
      if (selectedCategory()) q.set("category", selectedCategory());
      if (selectedOwner()) q.set("owner_id", selectedOwner());

      const res = await api.get<ProjectListItem[]>(`/api/admin/projects?${q.toString()}`);
      setProjects(res);
    } catch (e) {
      console.error("Failed to fetch projects:", e);
    } finally {
      setLoading(false);
    }
  };

  const fetchMeta = async () => {
    try {
      const res = await api.get<MetaOptionsResponse>("/api/admin/meta/options");
      setMeta(res);
    } catch (e) {
      console.error("Failed to fetch meta options:", e);
    }
  };

  onMount(() => {
    fetchMeta();
    fetchProjects();
  });

  const handleCreateProject = async (e: Event) => {
    e.preventDefault();
    if (!formName().trim() || !formOwnerId() || !formLocationId()) {
      setFormError("Please fill in project name, owner, and location.");
      return;
    }

    setSubmitting(true);
    setFormError(null);

    try {
      await api.post("/api/admin/projects", {
        name: formName().trim(),
        project_owner_id: Number(formOwnerId()),
        location_id: Number(formLocationId()),
        category: formCategory(),
        description: formDesc() ? formDesc().trim() : undefined,
        launch_date: formLaunchDate() || undefined,
        possession_date: formPossessionDate() || undefined,
        amenity_ids: formAmenityIds(),
      });

      setModalOpen(false);
      resetForm();
      fetchProjects();
    } catch (err: any) {
      setFormError(err.message || "Failed to create project");
    } finally {
      setSubmitting(false);
    }
  };

  const resetForm = () => {
    setFormName("");
    setFormOwnerId("");
    setFormLocationId("");
    setFormCategory("RESIDENTIAL");
    setFormDesc("");
    setFormLaunchDate("");
    setFormPossessionDate("");
    setFormAmenityIds([]);
    setFormError(null);
  };

  const toggleAmenity = (id: number) => {
    if (formAmenityIds().includes(id)) {
      setFormAmenityIds(formAmenityIds().filter((a) => a !== id));
    } else {
      setFormAmenityIds([...formAmenityIds(), id]);
    }
  };

  return (
    <div class="space-y-6">
      {/* Header */}
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 pb-4 border-b border-slate-800">
        <div>
          <h1 class="text-2xl font-bold tracking-tight text-white">Projects Directory</h1>
          <p class="text-xs text-slate-400 mt-1">
            Manage real estate developments, towers, master plans, and unit allocations
          </p>
        </div>

        <Button onClick={() => setModalOpen(true)}>
          <Plus class="w-4 h-4" /> New Development Project
        </Button>
      </div>

      {/* Filter and Search Bar */}
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
        <div class="relative">
          <Input
            placeholder="Search projects..."
            value={search()}
            onInput={(e) => {
              setSearch(e.currentTarget.value);
              debouncedFetchProjects();
            }}
          />
        </div>

        <Select
          value={selectedCategory()}
          onChange={(e) => {
            setSelectedCategory(e.currentTarget.value);
            fetchProjects();
          }}
          options={[
            { value: "", label: "All Categories" },
            { value: "RESIDENTIAL", label: "Residential" },
            { value: "COMMERCIAL", label: "Commercial" },
            { value: "LAND", label: "Plots & Land" },
          ]}
        />

        <Select
          value={selectedOwner()}
          onChange={(e) => {
            setSelectedOwner(e.currentTarget.value);
            fetchProjects();
          }}
          options={[
            { value: "", label: "All Project Owners" },
            ...(meta()?.owners.map((o) => ({ value: o.id, label: o.name })) || []),
          ]}
        />
      </div>

      {/* Projects Grid */}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
        <For each={projects()}>
          {(proj) => (
            <div class="bg-slate-900 border border-slate-800 rounded-2xl overflow-hidden hover:border-slate-700 transition-all flex flex-col group shadow-sm">
              {/* Thumbnail Hero */}
              <div class="aspect-video bg-slate-950 relative overflow-hidden flex items-center justify-center">
                {proj.thumbnail_url ? (
                  <img
                    src={proj.thumbnail_url}
                    alt={proj.name}
                    class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                  />
                ) : (
                  <div class="flex flex-col items-center justify-center text-slate-600">
                    <Building2 class="w-10 h-10 mb-2 opacity-50" />
                    <span class="text-xs font-mono uppercase tracking-wider">No Media</span>
                  </div>
                )}
                <div class="absolute top-3 left-3">
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
                </div>
              </div>

              {/* Body */}
              <div class="p-5 flex-1 flex flex-col justify-between">
                <div>
                  <h3 class="text-base font-bold text-white group-hover:text-indigo-400 transition-colors">
                    {proj.name}
                  </h3>
                  <p class="text-xs font-medium text-slate-400 mt-0.5">by {proj.owner_name}</p>

                  <div class="flex items-center gap-1.5 text-xs text-slate-400 mt-3">
                    <MapPin class="w-3.5 h-3.5 text-slate-500 shrink-0" />
                    <span class="truncate">{proj.location_name}</span>
                  </div>
                </div>

                <div class="flex items-center justify-between pt-4 mt-4 border-t border-slate-850">
                  <span class="text-xs text-slate-400">
                    <strong class="text-slate-100 font-mono text-sm">{proj.properties_count}</strong> units listed
                  </span>

                  <A href={`/projects/${proj.id}`}>
                    <Button variant="secondary" size="sm">
                      Manage <ExternalLink class="w-3 h-3 ml-1" />
                    </Button>
                  </A>
                </div>
              </div>
            </div>
          )}
        </For>
      </div>

      <Show when={projects().length === 0 && !loading()}>
        <div class="text-center py-16 bg-slate-900/40 border border-slate-800 rounded-2xl">
          <Building2 class="w-10 h-10 mx-auto text-slate-600 mb-2" />
          <p class="text-sm font-medium text-slate-300">No projects found</p>
          <p class="text-xs text-slate-500 mt-1">Try clearing filters or create a new development project.</p>
        </div>
      </Show>

      {/* Create Project Modal */}
      <Modal
        isOpen={modalOpen()}
        onClose={() => setModalOpen(false)}
        title="Register New Development Project"
        description="Add a new master development or commercial complex to the platform."
        maxWidth="2xl"
        footer={
          <>
            <Button variant="ghost" onClick={() => setModalOpen(false)}>
              Cancel
            </Button>
            <Button loading={submitting()} onClick={handleCreateProject}>
              Create Project
            </Button>
          </>
        }
      >
        <form onSubmit={handleCreateProject} class="space-y-4">
          {formError() && (
            <div class="p-3 bg-rose-950/60 border border-rose-800 rounded-lg text-xs text-rose-300">
              {formError()}
            </div>
          )}

          <Input
            label="Project Name"
            required
            placeholder="e.g. Skyline Heights Residencies"
            value={formName()}
            onInput={(e) => setFormName(e.currentTarget.value)}
          />

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <Select
              label="Project Owner / Developer"
              required
              placeholder="Select developer"
              value={formOwnerId()}
              onChange={(e) => setFormOwnerId(Number(e.currentTarget.value))}
              options={meta()?.owners.map((o) => ({ value: o.id, label: o.name })) || []}
            />

            <Select
              label="Location"
              required
              placeholder="Select location"
              value={formLocationId()}
              onChange={(e) => setFormLocationId(Number(e.currentTarget.value))}
              options={meta()?.locations.map((l) => ({ value: l.id, label: l.formatted_address })) || []}
            />
          </div>

          <Select
            label="Development Category"
            required
            value={formCategory()}
            onChange={(e) => setFormCategory(e.currentTarget.value as any)}
            options={[
              { value: "RESIDENTIAL", label: "Residential (Villas, Apartments, Condos)" },
              { value: "COMMERCIAL", label: "Commercial (Offices, Retail, Workspaces)" },
              { value: "LAND", label: "Land & Plots (Residential / Commercial Plots)" },
            ]}
          />

          <Textarea
            label="Project Description"
            placeholder="Overview of the development, architecture highlights, landmark connectivity..."
            value={formDesc()}
            onInput={(e) => setFormDesc(e.currentTarget.value)}
          />

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <Input
              label="Launch Date"
              type="date"
              value={formLaunchDate()}
              onInput={(e) => setFormLaunchDate(e.currentTarget.value)}
            />
            <Input
              label="Possession Date"
              type="date"
              value={formPossessionDate()}
              onInput={(e) => setFormPossessionDate(e.currentTarget.value)}
            />
          </div>

          {/* Amenities Multi-Check */}
          <div>
            <label class="block text-xs font-medium text-slate-300 mb-2">
              Project Amenities
            </label>
            <div class="grid grid-cols-2 sm:grid-cols-3 gap-2 max-h-36 overflow-y-auto p-2 bg-slate-950 border border-slate-800 rounded-lg">
              <For each={meta()?.amenities}>
                {(amenity) => (
                  <label class="flex items-center gap-2 text-xs text-slate-300 cursor-pointer p-1 hover:bg-slate-900 rounded">
                    <input
                      type="checkbox"
                      checked={formAmenityIds().includes(amenity.id)}
                      onChange={() => toggleAmenity(amenity.id)}
                      class="rounded border-slate-700 bg-slate-900 text-indigo-600 focus:ring-indigo-500/20"
                    />
                    <span class="truncate">{amenity.name}</span>
                  </label>
                )}
              </For>
            </div>
          </div>
        </form>
      </Modal>
    </div>
  );
}

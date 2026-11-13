import { createSignal, onMount, For, Show } from "solid-js";
import { A, useSearchParams } from "@solidjs/router";
import { Building2, Plus, MapPin, ShieldCheck, Layers } from "lucide-solid";
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
  const [selectedType, setSelectedType] = createSignal("");
  const [selectedOwner, setSelectedOwner] = createSignal("");

  const debouncedFetchProjects = debounce(() => {
    fetchProjects();
  }, 300);

  // Create Project Modal state
  const [modalOpen, setModalOpen] = createSignal(params.action === "new");
  const [submitting, setSubmitting] = createSignal(false);
  const [formError, setFormError] = createSignal<string | null>(null);

  // Form fields
  const [formName, setFormName] = createSignal("");
  const [formOwnerId, setFormOwnerId] = createSignal<number | "">("");
  const [formProjectTypeId, setFormProjectTypeId] = createSignal<number | "">("");
  const [formFormattedAddress, setFormFormattedAddress] = createSignal("");
  const [formCity, setFormCity] = createSignal("");
  const [formArea, setFormArea] = createSignal("");
  const [formState, setFormState] = createSignal("");
  const [formPostalCode, setFormPostalCode] = createSignal("");
  const [formLatitude, setFormLatitude] = createSignal<number | "">("");
  const [formLongitude, setFormLongitude] = createSignal<number | "">("");
  const [formReraNumber, setFormReraNumber] = createSignal("");
  const [formReraAuthority, setFormReraAuthority] = createSignal("");
  const [formTotalLandArea, setFormTotalLandArea] = createSignal<number | "">("");
  const [formTowersCount, setFormTowersCount] = createSignal<number | "">("");
  const [formFloorsCount, setFormFloorsCount] = createSignal<number | "">("");
  const [formUnitsCount, setFormUnitsCount] = createSignal<number | "">("");
  const [formDesc, setFormDesc] = createSignal("");
  const [formStartDate, setFormStartDate] = createSignal("");
  const [formLaunchDate, setFormLaunchDate] = createSignal("");
  const [formPossessionDate, setFormPossessionDate] = createSignal("");
  const [formAmenityIds, setFormAmenityIds] = createSignal<number[]>([]);

  const fetchProjects = async () => {
    setLoading(true);
    try {
      const q = new URLSearchParams();
      if (search().trim()) q.set("q", search().trim());
      if (selectedType()) q.set("project_type_id", selectedType());
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
    if (!formName().trim() || !formOwnerId() || !formProjectTypeId()) {
      setFormError("Please fill in project name, developer, and project type.");
      return;
    }
    if (!formCity().trim() || !formArea().trim()) {
      setFormError("Please provide both city and area for the embedded location.");
      return;
    }

    setSubmitting(true);
    setFormError(null);

    try {
      await api.post("/api/admin/projects", {
        name: formName().trim(),
        project_owner_id: Number(formOwnerId()),
        project_type_id: Number(formProjectTypeId()),
        description: formDesc().trim() || null,
        formatted_address: formFormattedAddress().trim() || `${formArea().trim()}, ${formCity().trim()}`,
        city: formCity().trim(),
        area: formArea().trim(),
        state_or_province: formState().trim() || "Gujarat",
        postal_code: formPostalCode().trim() || null,
        latitude: formLatitude() !== "" ? Number(formLatitude()) : null,
        longitude: formLongitude() !== "" ? Number(formLongitude()) : null,
        rera_number: formReraNumber().trim() || null,
        rera_authority: formReraAuthority().trim() || null,
        total_land_area: formTotalLandArea() !== "" ? Number(formTotalLandArea()) : null,
        towers_count: formTowersCount() !== "" ? Number(formTowersCount()) : null,
        floors_count: formFloorsCount() !== "" ? Number(formFloorsCount()) : null,
        units_count: formUnitsCount() !== "" ? Number(formUnitsCount()) : null,
        start_date: formStartDate() || null,
        launch_date: formLaunchDate() || null,
        possession_date: formPossessionDate() || null,
        amenities: formAmenityIds(),
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
    setFormProjectTypeId("");
    setFormFormattedAddress("");
    setFormCity("");
    setFormArea("");
    setFormState("");
    setFormPostalCode("");
    setFormLatitude("");
    setFormLongitude("");
    setFormReraNumber("");
    setFormReraAuthority("");
    setFormTotalLandArea("");
    setFormTowersCount("");
    setFormFloorsCount("");
    setFormUnitsCount("");
    setFormDesc("");
    setFormStartDate("");
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
      {/* Datalist registries for Creatable inputs */}
      <datalist id="cities-datalist">
        <For each={meta()?.cities}>{(c) => <option value={c} />}</For>
      </datalist>
      <datalist id="areas-datalist">
        <For each={meta()?.areas}>{(a) => <option value={a} />}</For>
      </datalist>
      <datalist id="states-datalist">
        <For each={meta()?.states}>{(s) => <option value={s} />}</For>
      </datalist>

      {/* Header */}
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 pb-4 border-b border-slate-800">
        <div>
          <h1 class="text-2xl font-bold tracking-tight text-white">Master Projects</h1>
          <p class="text-xs text-slate-400 mt-1">
            Manage real estate developments, towers, floor plans, specifications, and connectivity
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
            placeholder="Search projects, locality..."
            value={search()}
            onInput={(e) => {
              setSearch(e.currentTarget.value);
              debouncedFetchProjects();
            }}
          />
        </div>

        <Select
          value={selectedType()}
          onChange={(e) => {
            setSelectedType(e.currentTarget.value);
            fetchProjects();
          }}
          options={[
            { value: "", label: "All Project Types" },
            ...(meta()?.project_types.map((t) => ({ value: t.id, label: t.name })) || []),
          ]}
        />

        <Select
          value={selectedOwner()}
          onChange={(e) => {
            setSelectedOwner(e.currentTarget.value);
            fetchProjects();
          }}
          options={[
            { value: "", label: "All Developers / Owners" },
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
                <div class="absolute top-3 left-3 flex gap-2">
                  <Badge variant="default">
                    {proj.project_type_name}
                  </Badge>
                  {proj.rera_number && (
                    <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-950/80 text-emerald-400 border border-emerald-800/60 backdrop-blur-sm">
                      <ShieldCheck class="w-3 h-3" /> RERA
                    </span>
                  )}
                </div>
              </div>

              {/* Body */}
              <div class="p-5 flex-1 flex flex-col justify-between">
                <div>
                  <h3 class="text-base font-bold text-white group-hover:text-indigo-400 transition-colors">
                    {proj.name}
                  </h3>
                  <p class="text-xs font-medium text-slate-400 mt-0.5">by {proj.owner_name}</p>

                  <div class="flex items-center gap-1 text-xs text-slate-400 mt-3">
                    <MapPin class="w-3.5 h-3.5 text-slate-500 shrink-0" />
                    <span class="truncate">{proj.area}, {proj.city}</span>
                  </div>
                </div>

                <div class="mt-4 pt-4 border-t border-slate-800/80 flex items-center justify-between">
                  <div class="text-xs text-slate-400 flex items-center gap-1.5 font-medium">
                    <Layers class="w-3.5 h-3.5 text-indigo-400" />
                    <span>{proj.configurations_count} Configurations</span>
                  </div>

                  <div class="flex items-center gap-2">
                    <A
                      href={`/projects/${proj.id}`}
                      class="text-xs font-semibold text-indigo-400 hover:text-indigo-300 transition-colors"
                    >
                      Manage &rarr;
                    </A>
                  </div>
                </div>
              </div>
            </div>
          )}
        </For>
      </div>

      <Show when={!loading() && projects().length === 0}>
        <div class="p-12 text-center border border-dashed border-slate-800 rounded-2xl bg-slate-900/40">
          <Building2 class="w-10 h-10 text-slate-600 mx-auto mb-3" />
          <h3 class="text-sm font-semibold text-slate-300">No projects found</h3>
          <p class="text-xs text-slate-500 mt-1 max-w-sm mx-auto">
            Try adjusting your search criteria or add your first master development project.
          </p>
          <Button class="mt-4" onClick={() => setModalOpen(true)}>
            <Plus class="w-4 h-4" /> Create Project
          </Button>
        </div>
      </Show>

      {/* Create Project Modal */}
      <Modal
        isOpen={modalOpen()}
        onClose={() => setModalOpen(false)}
        title="Create Master Development Project"
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
        <form onSubmit={handleCreateProject} class="space-y-4 max-h-[75vh] overflow-y-auto pr-2">
          {formError() && (
            <div class="p-3 bg-rose-950/60 border border-rose-800 rounded-lg text-xs text-rose-300">
              {formError()}
            </div>
          )}

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Input
              label="Project Title"
              required
              placeholder="e.g. Prestige Eden Park"
              value={formName()}
              onInput={(e) => setFormName(e.currentTarget.value)}
            />

            <Select
              label="Developer / Owner"
              required
              value={formOwnerId()}
              onChange={(e) => setFormOwnerId(Number(e.currentTarget.value))}
              options={[
                { value: "", label: "Select Developer..." },
                ...(meta()?.owners.map((o) => ({ value: o.id, label: o.name })) || []),
              ]}
            />
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Select
              label="Project Type"
              required
              value={formProjectTypeId()}
              onChange={(e) => setFormProjectTypeId(Number(e.currentTarget.value))}
              options={[
                { value: "", label: "Select Project Type..." },
                ...(meta()?.project_types.map((t) => ({ value: t.id, label: t.name })) || []),
              ]}
            />

            <Input
              label="Total Land Area (sq ft)"
              type="number"
              placeholder="e.g. 150000"
              value={formTotalLandArea() !== "" ? String(formTotalLandArea()) : ""}
              onInput={(e) => setFormTotalLandArea(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
            />
          </div>

          {/* Embedded Location Header */}
          <div class="pt-2 border-t border-slate-800">
            <h4 class="text-xs font-bold text-slate-300 uppercase tracking-wider mb-2 flex items-center gap-1.5">
              <MapPin class="w-3.5 h-3.5 text-indigo-400" /> Embedded Location
            </h4>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
            <div>
              <label class="block text-xs font-medium text-slate-300 mb-1">City *</label>
              <input
                list="cities-datalist"
                required
                class="w-full px-3 py-2 bg-slate-900 border border-slate-800 rounded-lg text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
                placeholder="e.g. Ahmedabad"
                value={formCity()}
                onInput={(e) => setFormCity(e.currentTarget.value)}
              />
            </div>

            <div>
              <label class="block text-xs font-medium text-slate-300 mb-1">Area / Locality *</label>
              <input
                list="areas-datalist"
                required
                class="w-full px-3 py-2 bg-slate-900 border border-slate-800 rounded-lg text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
                placeholder="e.g. Bodakdev"
                value={formArea()}
                onInput={(e) => setFormArea(e.currentTarget.value)}
              />
            </div>

            <div>
              <label class="block text-xs font-medium text-slate-300 mb-1">State</label>
              <input
                list="states-datalist"
                class="w-full px-3 py-2 bg-slate-900 border border-slate-800 rounded-lg text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
                placeholder="e.g. Gujarat"
                value={formState()}
                onInput={(e) => setFormState(e.currentTarget.value)}
              />
            </div>
          </div>

          <Input
            label="Full Formatted Address"
            placeholder="e.g. Off Sindhu Bhavan Road, Bodakdev, Ahmedabad - 380054"
            value={formFormattedAddress()}
            onInput={(e) => setFormFormattedAddress(e.currentTarget.value)}
          />

          <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
            <Input
              label="Postal Code"
              placeholder="e.g. 380054"
              value={formPostalCode()}
              onInput={(e) => setFormPostalCode(e.currentTarget.value)}
            />
            <Input
              label="Latitude (GPS)"
              type="number"
              step="0.000001"
              placeholder="e.g. 23.0338"
              value={formLatitude() !== "" ? String(formLatitude()) : ""}
              onInput={(e) => setFormLatitude(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
            />
            <Input
              label="Longitude (GPS)"
              type="number"
              step="0.000001"
              placeholder="e.g. 72.5850"
              value={formLongitude() !== "" ? String(formLongitude()) : ""}
              onInput={(e) => setFormLongitude(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
            />
          </div>

          {/* Scale & Regulatory */}
          <div class="pt-2 border-t border-slate-800">
            <h4 class="text-xs font-bold text-slate-300 uppercase tracking-wider mb-2 flex items-center gap-1.5">
              <ShieldCheck class="w-3.5 h-3.5 text-indigo-400" /> RERA & Scale
            </h4>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
            <Input
              label="RERA Registration Number"
              placeholder="e.g. PR/GJ/AHMEDABAD/..."
              value={formReraNumber()}
              onInput={(e) => setFormReraNumber(e.currentTarget.value)}
            />
            <Input
              label="RERA Authority"
              placeholder="e.g. Gujarat RERA (GujRERA)"
              value={formReraAuthority()}
              onInput={(e) => setFormReraAuthority(e.currentTarget.value)}
            />
          </div>

          <div class="grid grid-cols-3 gap-3">
            <Input
              label="Towers"
              type="number"
              placeholder="e.g. 4"
              value={formTowersCount() !== "" ? String(formTowersCount()) : ""}
              onInput={(e) => setFormTowersCount(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
            />
            <Input
              label="Floors"
              type="number"
              placeholder="e.g. 22"
              value={formFloorsCount() !== "" ? String(formFloorsCount()) : ""}
              onInput={(e) => setFormFloorsCount(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
            />
            <Input
              label="Units"
              type="number"
              placeholder="e.g. 180"
              value={formUnitsCount() !== "" ? String(formUnitsCount()) : ""}
              onInput={(e) => setFormUnitsCount(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
            />
          </div>

          {/* Dates */}
          <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
            <Input
              label="Start Date"
              type="date"
              value={formStartDate()}
              onInput={(e) => setFormStartDate(e.currentTarget.value)}
            />
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

          <Textarea
            label="Project Description (Markdown supported)"
            rows={3}
            placeholder="Describe master plan architectural highlights, landscape spaces, luxury finishes..."
            value={formDesc()}
            onInput={(e) => setFormDesc(e.currentTarget.value)}
          />

          {/* Amenities Multi-select */}
          <div>
            <label class="block text-xs font-medium text-slate-300 mb-2">
              Select Amenities
            </label>
            <div class="grid grid-cols-2 sm:grid-cols-3 gap-2 max-h-40 overflow-y-auto p-2 border border-slate-800 rounded-xl bg-slate-950">
              <For each={meta()?.amenities}>
                {(amenity) => (
                  <label class="flex items-center gap-2 p-1.5 rounded-lg hover:bg-slate-900 cursor-pointer text-xs text-slate-300 transition-colors">
                    <input
                      type="checkbox"
                      checked={formAmenityIds().includes(amenity.id)}
                      onChange={() => toggleAmenity(amenity.id)}
                      class="rounded border-slate-700 bg-slate-800 text-indigo-600 focus:ring-indigo-500 h-3.5 w-3.5"
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

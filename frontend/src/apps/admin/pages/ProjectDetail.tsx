import { createSignal, onMount, For } from "solid-js";
import { useParams, useSearchParams, A } from "@solidjs/router";
import {
  MapPin,
  Building2,
  Plus,
  Trash2,
  ExternalLink,
  ArrowLeft,
  Upload,
  Edit,
  Save,
  Layers,
  Sparkles,
  Compass,
  FileCheck2,
  Image as ImageIcon,
  Check,
  X,
} from "lucide-solid";

import { Button } from "@/shared/components/Button";
import { Badge } from "@/shared/components/Badge";
import { Modal } from "@/shared/components/Modal";
import { Input } from "@/shared/components/Input";
import { Select } from "@/shared/components/Select";
import { Textarea } from "@/shared/components/Textarea";
import { MediaManager } from "@/apps/admin/components/MediaManager";
import { api } from "@/shared/api/client";
import {
  ProjectDetailData,
  MetaOptionsResponse,
  ProjectConfigurationItem,
} from "@/shared/api/types";

type TabKey = "overview" | "location" | "configurations" | "specifications" | "landmarks" | "media" | "documents";

export function ProjectDetail() {
  const params = useParams();
  const [searchParams, setSearchParams] = useSearchParams();
  const projectId = () => Number(params.id);

  const [project, setProject] = createSignal<ProjectDetailData | null>(null);
  const [meta, setMeta] = createSignal<MetaOptionsResponse | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [saving, setSaving] = createSignal(false);
  const [saveSuccess, setSaveSuccess] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const activeTab = (): TabKey => {
    const rawTab = searchParams.tab;
    const tab = (Array.isArray(rawTab) ? rawTab[0] : rawTab)?.toLowerCase();
    if (tab === "location") return "location";
    if (tab === "configurations" || tab === "configs" || tab === "units") return "configurations";
    if (tab === "specifications" || tab === "specs") return "specifications";
    if (tab === "landmarks" || tab === "connectivity") return "landmarks";
    if (tab === "media" || tab === "gallery") return "media";
    if (tab === "documents" || tab === "docs") return "documents";
    return "overview";
  };

  const selectTab = (tab: TabKey) => {
    setSearchParams({ tab });
  };

  // ---------------------------------------------------------------------------
  // Tab 1 & 2 Form State (Project Overview & Embedded Location)
  // ---------------------------------------------------------------------------
  const [formName, setFormName] = createSignal("");
  const [formOwnerId, setFormOwnerId] = createSignal<number | "">("");
  const [formProjectTypeId, setFormProjectTypeId] = createSignal<number | "">("");
  const [formDesc, setFormDesc] = createSignal("");
  const [formStartDate, setFormStartDate] = createSignal("");
  const [formLaunchDate, setFormLaunchDate] = createSignal("");
  const [formPossessionDate, setFormPossessionDate] = createSignal("");

  // Location fields
  const [formFormattedAddress, setFormFormattedAddress] = createSignal("");
  const [formCity, setFormCity] = createSignal("");
  const [formArea, setFormArea] = createSignal("");
  const [formState, setFormState] = createSignal("");
  const [formPostalCode, setFormPostalCode] = createSignal("");
  const [formLatitude, setFormLatitude] = createSignal<number | "">("");
  const [formLongitude, setFormLongitude] = createSignal<number | "">("");

  // Regulatory & scale
  const [formReraNumber, setFormReraNumber] = createSignal("");
  const [formReraAuthority, setFormReraAuthority] = createSignal("");
  const [formTotalLandArea, setFormTotalLandArea] = createSignal<number | "">("");
  const [formTowersCount, setFormTowersCount] = createSignal<number | "">("");
  const [formFloorsCount, setFormFloorsCount] = createSignal<number | "">("");
  const [formUnitsCount, setFormUnitsCount] = createSignal<number | "">("");
  const [formAmenityIds, setFormAmenityIds] = createSignal<number[]>([]);

  // ---------------------------------------------------------------------------
  // Tab 3: Configurations & Floor Plans State
  // ---------------------------------------------------------------------------
  const [configModalOpen, setConfigModalOpen] = createSignal(false);
  const [editingConfigId, setEditingConfigId] = createSignal<number | null>(null);
  const [configName, setConfigName] = createSignal("");
  const [configBhk, setConfigBhk] = createSignal<number | "">("");
  const [configCarpetArea, setConfigCarpetArea] = createSignal<number | "">("");
  const [configBuiltUpArea, setConfigBuiltUpArea] = createSignal<number | "">("");
  const [configSuperBuiltUpArea, setConfigSuperBuiltUpArea] = createSignal<number | "">("");
  const [configPrice, setConfigPrice] = createSignal<number | "">("");
  const [configPriceOnRequest, setConfigPriceOnRequest] = createSignal(false);
  const [configFloorPlanFile, setConfigFloorPlanFile] = createSignal<File | null>(null);
  const [configSubmitting, setConfigSubmitting] = createSignal(false);

  // Master Floor Plans state
  const [floorPlanModalOpen, setFloorPlanModalOpen] = createSignal(false);
  const [floorPlanTitle, setFloorPlanTitle] = createSignal("");
  const [floorPlanFile, setFloorPlanFile] = createSignal<File | null>(null);
  const [floorPlanSubmitting, setFloorPlanSubmitting] = createSignal(false);

  // ---------------------------------------------------------------------------
  // Tab 4: Specifications State
  // ---------------------------------------------------------------------------
  const [specs, setSpecs] = createSignal<Array<{ category: string; items: string[] }>>([]);
  const [newSpecCategory, setNewSpecCategory] = createSignal("");

  // ---------------------------------------------------------------------------
  // Tab 5: Landmarks State
  // ---------------------------------------------------------------------------
  const [landmarks, setLandmarks] = createSignal<Array<{
    category: string;
    name: string;
    distance: string;
    duration: string;
    latitude: number | null;
    longitude: number | null;
    source: string;
  }>>([]);
  const [landmarkModalOpen, setLandmarkModalOpen] = createSignal(false);
  const [lmCategory, setLmCategory] = createSignal("Connectivity");
  const [lmName, setLmName] = createSignal("");
  const [lmDistance, setLmDistance] = createSignal("");
  const [lmDuration, setLmDuration] = createSignal("");
  const [lmLat, setLmLat] = createSignal<number | "">("");
  const [lmLng, setLmLng] = createSignal<number | "">("");

  // ---------------------------------------------------------------------------
  // Tab 7: Documents State
  // ---------------------------------------------------------------------------
  const [docModalOpen, setDocModalOpen] = createSignal(false);
  const [docDisplayName, setDocDisplayName] = createSignal("");
  const [docType, setDocType] = createSignal("brochure");
  const [docFile, setDocFile] = createSignal<File | null>(null);
  const [docSubmitting, setDocSubmitting] = createSignal(false);

  // Inline Quick Add Amenity Modal
  const [amenityModalOpen, setAmenityModalOpen] = createSignal(false);
  const [quickAmenityName, setQuickAmenityName] = createSignal("");
  const [quickAmenityLoading, setQuickAmenityLoading] = createSignal(false);

  const fetchProject = async () => {
    setLoading(true);
    try {
      const data = await api.get<ProjectDetailData>(`/api/admin/projects/${projectId()}`);
      setProject(data);

      // Populate form
      setFormName(data.name);
      setFormOwnerId(data.owner_id);
      setFormProjectTypeId(data.project_type_id);
      setFormDesc(data.description || "");
      setFormStartDate(data.start_date || "");
      setFormLaunchDate(data.launch_date || "");
      setFormPossessionDate(data.possession_date || "");

      setFormFormattedAddress(data.formatted_address || "");
      setFormCity(data.city || "");
      setFormArea(data.area || "");
      setFormState(data.state_or_province || "");
      setFormPostalCode(data.postal_code || "");
      setFormLatitude(data.latitude ?? "");
      setFormLongitude(data.longitude ?? "");

      setFormReraNumber(data.rera_number || "");
      setFormReraAuthority(data.rera_authority || "");
      setFormTotalLandArea(data.total_land_area ?? "");
      setFormTowersCount(data.towers_count ?? "");
      setFormFloorsCount(data.floors_count ?? "");
      setFormUnitsCount(data.units_count ?? "");
      setFormAmenityIds(data.amenity_ids || []);

      // Specifications
      setSpecs(
        data.specifications.map((s) => ({
          category: s.category,
          items: [...s.items],
        }))
      );

      // Landmarks
      setLandmarks(
        data.landmarks.map((l) => ({
          category: l.category,
          name: l.name,
          distance: l.distance || "",
          duration: l.duration || "",
          latitude: l.latitude ?? null,
          longitude: l.longitude ?? null,
          source: l.source,
        }))
      );
    } catch (e: any) {
      setError(e.message || "Failed to load project details");
    } finally {
      setLoading(false);
    }
  };

  const fetchMeta = async () => {
    try {
      const res = await api.get<MetaOptionsResponse>("/api/admin/meta/options");
      setMeta(res);
    } catch (e) {
      console.error(e);
    }
  };

  onMount(() => {
    fetchMeta();
    fetchProject();
  });

  // Save Project Core Form (Overview & Location)
  const handleSaveOverview = async (e?: Event) => {
    if (e) e.preventDefault();
    if (!formName().trim() || !formOwnerId() || !formProjectTypeId()) {
      alert("Project name, developer, and project type are required.");
      return;
    }
    if (!formCity().trim() || !formArea().trim()) {
      alert("City and Area are required for project embedded location.");
      return;
    }

    setSaving(true);
    setSaveSuccess(false);

    try {
      await api.put(`/api/admin/projects/${projectId()}`, {
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

      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
      fetchProject();
    } catch (err: any) {
      alert(err.message || "Failed to update project details");
    } finally {
      setSaving(false);
    }
  };

  // Quick Create Amenity
  const handleQuickAddAmenity = async () => {
    if (!quickAmenityName().trim()) return;
    setQuickAmenityLoading(true);
    try {
      const res = await api.post<{ id: number }>("/api/admin/amenities", {
        name: quickAmenityName().trim(),
        is_active: true,
      });
      await fetchMeta();
      if (res && res.id) {
        setFormAmenityIds([...formAmenityIds(), res.id]);
      }
      setQuickAmenityName("");
      setAmenityModalOpen(false);
    } catch (err: any) {
      alert(err.message || "Failed to add amenity");
    } finally {
      setQuickAmenityLoading(false);
    }
  };

  // ---------------------------------------------------------------------------
  // Configuration Handlers
  // ---------------------------------------------------------------------------
  const openNewConfigModal = () => {
    setEditingConfigId(null);
    setConfigName("");
    setConfigBhk("");
    setConfigCarpetArea("");
    setConfigBuiltUpArea("");
    setConfigSuperBuiltUpArea("");
    setConfigPrice("");
    setConfigPriceOnRequest(false);
    setConfigFloorPlanFile(null);
    setConfigModalOpen(true);
  };

  const openEditConfigModal = (cfg: ProjectConfigurationItem) => {
    setEditingConfigId(cfg.id);
    setConfigName(cfg.name);
    setConfigBhk(cfg.bhk ?? "");
    setConfigCarpetArea(cfg.carpet_area ?? "");
    setConfigBuiltUpArea(cfg.built_up_area ?? "");
    setConfigSuperBuiltUpArea(cfg.super_built_up_area ?? "");
    setConfigPrice(cfg.price ?? "");
    setConfigPriceOnRequest(cfg.price_on_request);
    setConfigFloorPlanFile(null);
    setConfigModalOpen(true);
  };

  const handleSaveConfig = async (e: Event) => {
    e.preventDefault();
    if (!configName().trim()) {
      alert("Configuration title is required");
      return;
    }

    setConfigSubmitting(true);
    try {
      const payload = {
        name: configName().trim(),
        bhk: configBhk() !== "" ? Number(configBhk()) : null,
        carpet_area: configCarpetArea() !== "" ? Number(configCarpetArea()) : null,
        built_up_area: configBuiltUpArea() !== "" ? Number(configBuiltUpArea()) : null,
        super_built_up_area: configSuperBuiltUpArea() !== "" ? Number(configSuperBuiltUpArea()) : null,
        price: configPrice() !== "" ? Number(configPrice()) : null,
        price_on_request: configPriceOnRequest(),
        floor_plan_media_id: null,
        sequence: 0,
        is_active: true,
      };

      let configId = editingConfigId();
      if (configId) {
        await api.put(`/api/admin/projects/${projectId()}/configurations/${configId}`, payload);
      } else {
        const res = await api.post<{ id: number }>(`/api/admin/projects/${projectId()}/configurations`, payload);
        configId = res.id;
      }

      // If a floor plan file was chosen, upload it 1:1
      if (configFloorPlanFile() && configId) {
        const formData = new FormData();
        formData.append("file", configFloorPlanFile()!);
        const { error: fpError } = await api.POST("/api/admin/projects/{id}/configurations/{cid}/floor-plan", {
          params: { path: { id: projectId(), cid: configId } },
          body: formData as any,
        });
        if (fpError) throw new Error("Failed to upload floor plan");
      }

      setConfigModalOpen(false);
      fetchProject();
    } catch (err: any) {
      alert(err.message || "Failed to save configuration");
    } finally {
      setConfigSubmitting(false);
    }
  };

  const handleDeleteConfig = async (cid: number) => {
    if (!confirm("Delete this configuration?")) return;
    try {
      await api.delete(`/api/admin/projects/${projectId()}/configurations/${cid}`);
      fetchProject();
    } catch (err: any) {
      alert(err.message || "Failed to delete configuration");
    }
  };

  // Master Floor Plans
  const handleUploadFloorPlan = async (e: Event) => {
    e.preventDefault();
    if (!floorPlanTitle().trim() || !floorPlanFile()) {
      alert("Please enter title and choose plan image");
      return;
    }

    setFloorPlanSubmitting(true);
    try {
      const formData = new FormData();
      formData.append("title", floorPlanTitle().trim());
      formData.append("file", floorPlanFile()!);
      await api.post(`/api/admin/projects/${projectId()}/floor-plans`, formData);

      setFloorPlanModalOpen(false);
      setFloorPlanTitle("");
      setFloorPlanFile(null);
      fetchProject();
    } catch (err: any) {
      alert(err.message || "Failed to upload floor plan");
    } finally {
      setFloorPlanSubmitting(false);
    }
  };

  const handleDeleteFloorPlan = async (fpid: string) => {
    if (!confirm("Delete this master floor plan?")) return;
    try {
      await api.delete(`/api/admin/projects/${projectId()}/floor-plans/${fpid}`);
      fetchProject();
    } catch (err: any) {
      alert(err.message || "Failed to delete floor plan");
    }
  };

  // ---------------------------------------------------------------------------
  // Specifications Handlers
  // ---------------------------------------------------------------------------
  const addSpecCategory = () => {
    const cat = newSpecCategory().trim();
    if (!cat) return;
    setSpecs([...specs(), { category: cat, items: [""] }]);
    setNewSpecCategory("");
  };

  const removeSpecCategory = (idx: number) => {
    setSpecs(specs().filter((_, i) => i !== idx));
  };

  const addSpecItem = (catIdx: number) => {
    const updated = [...specs()];
    updated[catIdx].items.push("");
    setSpecs(updated);
  };

  const removeSpecItem = (catIdx: number, itemIdx: number) => {
    const updated = [...specs()];
    updated[catIdx].items = updated[catIdx].items.filter((_, i) => i !== itemIdx);
    setSpecs(updated);
  };

  const updateSpecItem = (catIdx: number, itemIdx: number, val: string) => {
    const updated = [...specs()];
    updated[catIdx].items[itemIdx] = val;
    setSpecs(updated);
  };

  const handleSaveSpecs = async () => {
    setSaving(true);
    try {
      const payload = {
        specifications: specs()
          .filter((s) => s.category.trim() && s.items.length > 0)
          .map((s, idx) => ({
            category: s.category.trim(),
            items: s.items.map((i) => i.trim()).filter(Boolean),
            sequence: idx,
          })),
      };
      const { error: specError } = await api.PUT("/api/admin/projects/{id}/specifications", {
        params: { path: { id: projectId() } },
        body: payload,
      });
      if (specError) throw new Error("Failed to save specifications");
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
      fetchProject();
    } catch (err: any) {
      alert(err.message || "Failed to save specifications");
    } finally {
      setSaving(false);
    }
  };

  // ---------------------------------------------------------------------------
  // Landmarks Handlers
  // ---------------------------------------------------------------------------
  const handleAddLandmark = (e: Event) => {
    e.preventDefault();
    if (!lmName().trim()) return;

    setLandmarks([
      ...landmarks(),
      {
        category: lmCategory(),
        name: lmName().trim(),
        distance: lmDistance().trim(),
        duration: lmDuration().trim(),
        latitude: lmLat() !== "" ? Number(lmLat()) : null,
        longitude: lmLng() !== "" ? Number(lmLng()) : null,
        source: "manual",
      },
    ]);

    setLmName("");
    setLmDistance("");
    setLmDuration("");
    setLmLat("");
    setLmLng("");
    setLandmarkModalOpen(false);
  };

  const removeLandmark = (idx: number) => {
    setLandmarks(landmarks().filter((_, i) => i !== idx));
  };

  const handleSaveLandmarks = async () => {
    setSaving(true);
    try {
      const payload = {
        landmarks: landmarks().map((l, idx) => ({
          category: l.category,
          name: l.name,
          distance: l.distance || null,
          duration: l.duration || null,
          latitude: l.latitude,
          longitude: l.longitude,
          source: l.source || "manual",
          sequence: idx,
        })),
      };
      const { error: lmError } = await api.PUT("/api/admin/projects/{id}/landmarks", {
        params: { path: { id: projectId() } },
        body: payload,
      });
      if (lmError) throw new Error("Failed to save landmarks");
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
      fetchProject();
    } catch (err: any) {
      alert(err.message || "Failed to save landmarks");
    } finally {
      setSaving(false);
    }
  };

  // ---------------------------------------------------------------------------
  // Documents Handlers
  // ---------------------------------------------------------------------------
  const handleUploadDocument = async (e: Event) => {
    e.preventDefault();
    if (!docDisplayName().trim() || !docFile()) {
      alert("Please provide title and PDF document file");
      return;
    }

    setDocSubmitting(true);
    try {
      const formData = new FormData();
      formData.append("display_name", docDisplayName().trim());
      formData.append("doc_type", docType());
      formData.append("file", docFile()!);
      await api.post(`/api/admin/projects/${projectId()}/documents`, formData);

      setDocModalOpen(false);
      setDocDisplayName("");
      setDocFile(null);
      fetchProject();
    } catch (err: any) {
      alert(err.message || "Failed to upload document");
    } finally {
      setDocSubmitting(false);
    }
  };

  const handleDeleteDocument = async (docId: string) => {
    if (!confirm("Delete this document?")) return;
    try {
      await api.delete(`/api/admin/projects/${projectId()}/documents/${docId}`);
      fetchProject();
    } catch (err: any) {
      alert(err.message || "Failed to delete document");
    }
  };

  const toggleAmenity = (id: number) => {
    if (formAmenityIds().includes(id)) {
      setFormAmenityIds(formAmenityIds().filter((a) => a !== id));
    } else {
      setFormAmenityIds([...formAmenityIds(), id]);
    }
  };

  return (
    <div class="space-y-6 pb-16">
      {/* Registries for Creatable inputs */}
      <datalist id="cities-datalist">
        <For each={meta()?.cities}>{(c) => <option value={c} />}</For>
      </datalist>
      <datalist id="areas-datalist">
        <For each={meta()?.areas}>{(a) => <option value={a} />}</For>
      </datalist>
      <datalist id="states-datalist">
        <For each={meta()?.states}>{(s) => <option value={s} />}</For>
      </datalist>

      {/* Top Breadcrumb & Actions Bar */}
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 pb-4 border-b border-slate-800">
        <div class="flex items-center gap-3">
          <A
            href="/projects"
            class="p-2 rounded-lg bg-slate-900 border border-slate-800 text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
          >
            <ArrowLeft class="w-4 h-4" />
          </A>
          <div>
            <div class="flex items-center gap-2">
              <h1 class="text-2xl font-bold tracking-tight text-white">
                {project()?.name || "Loading..."}
              </h1>
              {project()?.project_type_name && (
                <Badge variant="default">{project()?.project_type_name}</Badge>
              )}
            </div>
            <p class="text-xs text-slate-400 mt-0.5">
              by <span class="text-slate-200 font-semibold">{project()?.owner_name}</span> &bull; {project()?.area}, {project()?.city}
            </p>
            {error() && (
              <p class="text-xs text-rose-400 mt-1">{error()}</p>
            )}
            {loading() && !project() && (
              <p class="text-xs text-slate-500 mt-1 animate-pulse">Loading project details...</p>
            )}
          </div>
        </div>

        <div class="flex items-center gap-2">
          {saveSuccess() && (
            <span class="inline-flex items-center gap-1 text-xs text-emerald-400 font-medium bg-emerald-950/60 border border-emerald-800/80 px-2.5 py-1 rounded-lg">
              <Check class="w-3.5 h-3.5" /> Saved
            </span>
          )}
          <Button
            variant="secondary"
            size="sm"
            onClick={() => window.open(`/projects/${project()?.slug}`, "_blank")}
          >
            <ExternalLink class="w-3.5 h-3.5" /> View Public Page
          </Button>
          {(activeTab() === "overview" || activeTab() === "location") && (
            <Button size="sm" loading={saving()} onClick={() => handleSaveOverview()}>
              <Save class="w-3.5 h-3.5" /> Save Changes
            </Button>
          )}
          {activeTab() === "specifications" && (
            <Button size="sm" loading={saving()} onClick={handleSaveSpecs}>
              <Save class="w-3.5 h-3.5" /> Save Specifications
            </Button>
          )}
          {activeTab() === "landmarks" && (
            <Button size="sm" loading={saving()} onClick={handleSaveLandmarks}>
              <Save class="w-3.5 h-3.5" /> Save Landmarks
            </Button>
          )}
        </div>
      </div>

      {/* Sticky Tab Navigation Bar */}
      <div class="sticky top-0 z-20 bg-slate-950/90 backdrop-blur-md border-b border-slate-800 -mx-4 px-4 sm:-mx-6 sm:px-6">
        <nav class="flex items-center gap-1 overflow-x-auto py-2">
          <button
            onClick={() => selectTab("overview")}
            class={`flex items-center gap-2 px-3.5 py-2 rounded-lg text-xs font-semibold transition-all cursor-pointer shrink-0 ${
              activeTab() === "overview"
                ? "bg-indigo-600/15 text-indigo-400 border border-indigo-500/20"
                : "text-slate-400 hover:text-slate-200 hover:bg-slate-900"
            }`}
          >
            <Building2 class="w-3.5 h-3.5" />
            Overview & Developer
          </button>

          <button
            onClick={() => selectTab("location")}
            class={`flex items-center gap-2 px-3.5 py-2 rounded-lg text-xs font-semibold transition-all cursor-pointer shrink-0 ${
              activeTab() === "location"
                ? "bg-indigo-600/15 text-indigo-400 border border-indigo-500/20"
                : "text-slate-400 hover:text-slate-200 hover:bg-slate-900"
            }`}
          >
            <MapPin class="w-3.5 h-3.5" />
            Embedded Location
          </button>

          <button
            onClick={() => selectTab("configurations")}
            class={`flex items-center gap-2 px-3.5 py-2 rounded-lg text-xs font-semibold transition-all cursor-pointer shrink-0 ${
              activeTab() === "configurations"
                ? "bg-indigo-600/15 text-indigo-400 border border-indigo-500/20"
                : "text-slate-400 hover:text-slate-200 hover:bg-slate-900"
            }`}
          >
            <Layers class="w-3.5 h-3.5" />
            Configurations & Floor Plans ({project()?.configurations.length || 0})
          </button>

          <button
            onClick={() => selectTab("specifications")}
            class={`flex items-center gap-2 px-3.5 py-2 rounded-lg text-xs font-semibold transition-all cursor-pointer shrink-0 ${
              activeTab() === "specifications"
                ? "bg-indigo-600/15 text-indigo-400 border border-indigo-500/20"
                : "text-slate-400 hover:text-slate-200 hover:bg-slate-900"
            }`}
          >
            <Sparkles class="w-3.5 h-3.5" />
            Specifications
          </button>

          <button
            onClick={() => selectTab("landmarks")}
            class={`flex items-center gap-2 px-3.5 py-2 rounded-lg text-xs font-semibold transition-all cursor-pointer shrink-0 ${
              activeTab() === "landmarks"
                ? "bg-indigo-600/15 text-indigo-400 border border-indigo-500/20"
                : "text-slate-400 hover:text-slate-200 hover:bg-slate-900"
            }`}
          >
            <Compass class="w-3.5 h-3.5" />
            Landmarks & Connectivity ({landmarks().length})
          </button>

          <button
            onClick={() => selectTab("media")}
            class={`flex items-center gap-2 px-3.5 py-2 rounded-lg text-xs font-semibold transition-all cursor-pointer shrink-0 ${
              activeTab() === "media"
                ? "bg-indigo-600/15 text-indigo-400 border border-indigo-500/20"
                : "text-slate-400 hover:text-slate-200 hover:bg-slate-900"
            }`}
          >
            <ImageIcon class="w-3.5 h-3.5" />
            Media Gallery ({project()?.media.length || 0})
          </button>

          <button
            onClick={() => selectTab("documents")}
            class={`flex items-center gap-2 px-3.5 py-2 rounded-lg text-xs font-semibold transition-all cursor-pointer shrink-0 ${
              activeTab() === "documents"
                ? "bg-indigo-600/15 text-indigo-400 border border-indigo-500/20"
                : "text-slate-400 hover:text-slate-200 hover:bg-slate-900"
            }`}
          >
            <FileCheck2 class="w-3.5 h-3.5" />
            Compliance & Documents ({project()?.documents.length || 0})
          </button>
        </nav>
      </div>

      {/* --------------------------------------------------------------------- */}
      {/* TAB 1: OVERVIEW & DEVELOPER                                           */}
      {/* --------------------------------------------------------------------- */}
      {activeTab() === "overview" && (
        <form onSubmit={handleSaveOverview} class="space-y-6">
          <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 space-y-5">
            <h3 class="text-sm font-bold text-white uppercase tracking-wider flex items-center gap-2">
              <Building2 class="w-4 h-4 text-indigo-400" /> General Project Metadata
            </h3>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
              <Input
                label="Project Title"
                required
                value={formName()}
                onInput={(e) => setFormName(e.currentTarget.value)}
              />

              <Select
                label="Developer / Builder"
                required
                value={formOwnerId()}
                onChange={(e) => setFormOwnerId(Number(e.currentTarget.value))}
                options={[
                  { value: "", label: "Select Developer..." },
                  ...(meta()?.owners.map((o) => ({ value: o.id, label: o.name })) || []),
                ]}
              />

              <Select
                label="Project Primary Type"
                required
                value={formProjectTypeId()}
                onChange={(e) => setFormProjectTypeId(Number(e.currentTarget.value))}
                options={[
                  { value: "", label: "Select Project Type..." },
                  ...(meta()?.project_types.map((t) => ({ value: t.id, label: t.name })) || []),
                ]}
              />
            </div>

            <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
              <Input
                label="Total Land Area (sq ft)"
                type="number"
                value={formTotalLandArea() !== "" ? String(formTotalLandArea()) : ""}
                onInput={(e) => setFormTotalLandArea(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
              />
              <Input
                label="Towers Count"
                type="number"
                value={formTowersCount() !== "" ? String(formTowersCount()) : ""}
                onInput={(e) => setFormTowersCount(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
              />
              <Input
                label="Floors Count"
                type="number"
                value={formFloorsCount() !== "" ? String(formFloorsCount()) : ""}
                onInput={(e) => setFormFloorsCount(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
              />
              <Input
                label="Units Count"
                type="number"
                value={formUnitsCount() !== "" ? String(formUnitsCount()) : ""}
                onInput={(e) => setFormUnitsCount(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
              />
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <Input
                label="RERA Registration Number"
                placeholder="e.g. PR/GJ/AHMEDABAD/..."
                value={formReraNumber()}
                onInput={(e) => setFormReraNumber(e.currentTarget.value)}
              />
              <Input
                label="RERA Regulatory Authority"
                placeholder="e.g. Gujarat Real Estate Regulatory Authority"
                value={formReraAuthority()}
                onInput={(e) => setFormReraAuthority(e.currentTarget.value)}
              />
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
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
              label="Detailed Project Narrative (Markdown formatted)"
              rows={5}
              placeholder="Detailed architecture write-up, floor layouts, construction quality, master plan..."
              value={formDesc()}
              onInput={(e) => setFormDesc(e.currentTarget.value)}
            />
          </div>

          {/* Amenities Multi-Select with Quick Create */}
          <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <h3 class="text-sm font-bold text-white uppercase tracking-wider flex items-center gap-2">
                  <Sparkles class="w-4 h-4 text-indigo-400" /> Project Amenities
                </h3>
                <p class="text-xs text-slate-400 mt-0.5">
                  Select available lifestyle and infrastructure amenities
                </p>
              </div>

              <Button
                variant="secondary"
                size="sm"
                onClick={() => setAmenityModalOpen(true)}
              >
                <Plus class="w-3.5 h-3.5" /> Quick-Add Amenity
              </Button>
            </div>

            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2.5 max-h-56 overflow-y-auto p-3 border border-slate-800 rounded-xl bg-slate-950">
              <For each={meta()?.amenities}>
                {(amenity) => (
                  <label class="flex items-center gap-2 p-2 rounded-lg hover:bg-slate-900 cursor-pointer text-xs text-slate-300 transition-colors">
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
      )}

      {/* --------------------------------------------------------------------- */}
      {/* TAB 2: EMBEDDED LOCATION                                              */}
      {/* --------------------------------------------------------------------- */}
      {activeTab() === "location" && (
        <form onSubmit={handleSaveOverview} class="space-y-6">
          <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 space-y-5">
            <h3 class="text-sm font-bold text-white uppercase tracking-wider flex items-center gap-2">
              <MapPin class="w-4 h-4 text-indigo-400" /> Embedded Geographic Identity
            </h3>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
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
                <label class="block text-xs font-medium text-slate-300 mb-1">State / Province</label>
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
              label="Complete Formatted Address"
              placeholder="e.g. Off Sindhu Bhavan Road, Bodakdev, Ahmedabad - 380054"
              value={formFormattedAddress()}
              onInput={(e) => setFormFormattedAddress(e.currentTarget.value)}
            />

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
              <Input
                label="Postal / Zip Code"
                placeholder="e.g. 380054"
                value={formPostalCode()}
                onInput={(e) => setFormPostalCode(e.currentTarget.value)}
              />
              <Input
                label="Latitude (GPS Coordinates)"
                type="number"
                step="0.000001"
                placeholder="e.g. 23.033812"
                value={formLatitude() !== "" ? String(formLatitude()) : ""}
                onInput={(e) => setFormLatitude(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
              />
              <Input
                label="Longitude (GPS Coordinates)"
                type="number"
                step="0.000001"
                placeholder="e.g. 72.585023"
                value={formLongitude() !== "" ? String(formLongitude()) : ""}
                onInput={(e) => setFormLongitude(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
              />
            </div>

            {/* Interactive OpenStreetMap preview link */}
            {formLatitude() !== "" && formLongitude() !== "" && (
              <div class="p-4 bg-slate-950 border border-slate-800 rounded-xl flex items-center justify-between">
                <div class="flex items-center gap-2 text-xs text-slate-300">
                  <Compass class="w-4 h-4 text-emerald-400" />
                  <span>Geo-Coordinates: <strong>{formLatitude()}, {formLongitude()}</strong></span>
                </div>
                <a
                  href={`https://www.openstreetmap.org/?mlat=${formLatitude()}&mlon=${formLongitude()}#map=16/${formLatitude()}/${formLongitude()}`}
                  target="_blank"
                  rel="noopener"
                  class="text-xs font-semibold text-indigo-400 hover:underline flex items-center gap-1"
                >
                  View on OpenStreetMap <ExternalLink class="w-3 h-3" />
                </a>
              </div>
            )}
          </div>
        </form>
      )}

      {/* --------------------------------------------------------------------- */}
      {/* TAB 3: CONFIGURATIONS & FLOOR PLANS                                   */}
      {/* --------------------------------------------------------------------- */}
      {activeTab() === "configurations" && (
        <div class="space-y-8">
          {/* Configurations Typologies Table */}
          <div class="bg-slate-900 border border-slate-800 rounded-2xl overflow-hidden shadow-sm">
            <div class="flex items-center justify-between px-6 py-4 border-b border-slate-800">
              <div>
                <h3 class="text-sm font-bold text-white uppercase tracking-wider flex items-center gap-2">
                  <Layers class="w-4 h-4 text-indigo-400" /> BHK Typologies & Unit Plans
                </h3>
                <p class="text-xs text-slate-400 mt-0.5">
                  Exact areas, pricing, and 1:1 floor plan layout drawings
                </p>
              </div>

              <Button onClick={openNewConfigModal}>
                <Plus class="w-4 h-4" /> Add Configuration
              </Button>
            </div>

            <div class="overflow-x-auto">
              <table class="w-full text-left text-xs">
                <thead class="bg-slate-950/60 text-slate-400 uppercase tracking-wider font-semibold border-b border-slate-800">
                  <tr>
                    <th class="px-6 py-3">Typology / Title</th>
                    <th class="px-6 py-3">BHK</th>
                    <th class="px-6 py-3">Carpet Area</th>
                    <th class="px-6 py-3">Built-up</th>
                    <th class="px-6 py-3">Super Built-up</th>
                    <th class="px-6 py-3">Pricing</th>
                    <th class="px-6 py-3">Floor Plan</th>
                    <th class="px-6 py-3 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-slate-800/60">
                  <For
                    each={project()?.configurations}
                    fallback={
                      <tr>
                        <td colspan="8" class="px-6 py-12 text-center text-slate-500">
                          No configurations added yet. Click "Add Configuration" to create typologies.
                        </td>
                      </tr>
                    }
                  >
                    {(cfg) => (
                      <tr class="hover:bg-slate-800/40 transition-colors">
                        <td class="px-6 py-4 font-bold text-slate-100">{cfg.name}</td>
                        <td class="px-6 py-4">
                          {cfg.bhk != null ? <Badge variant="default">{cfg.bhk} BHK</Badge> : "—"}
                        </td>
                        <td class="px-6 py-4 font-mono text-slate-300">
                          {cfg.carpet_area != null ? `${cfg.carpet_area} sq.ft` : "—"}
                        </td>
                        <td class="px-6 py-4 font-mono text-slate-400">
                          {cfg.built_up_area != null ? `${cfg.built_up_area} sq.ft` : "—"}
                        </td>
                        <td class="px-6 py-4 font-mono text-slate-400">
                          {cfg.super_built_up_area != null ? `${cfg.super_built_up_area} sq.ft` : "—"}
                        </td>
                        <td class="px-6 py-4 font-semibold text-slate-200">
                          {cfg.price_on_request
                            ? "Price on Request"
                            : cfg.price != null
                            ? `₹ ${cfg.price.toLocaleString("en-IN")}`
                            : "—"}
                        </td>
                        <td class="px-6 py-4">
                          {cfg.floor_plan_url ? (
                            <a
                              href={cfg.floor_plan_url}
                              target="_blank"
                              rel="noopener"
                              class="inline-flex items-center gap-1.5 text-indigo-400 hover:underline"
                            >
                              <img
                                src={cfg.floor_plan_thumb_url || cfg.floor_plan_url}
                                alt="Floor plan"
                                class="w-8 h-8 rounded object-cover border border-slate-700"
                              />
                              <span>View Plan</span>
                            </a>
                          ) : (
                            <span class="text-slate-600">No Image</span>
                          )}
                        </td>
                        <td class="px-6 py-4 text-right space-x-2">
                          <button
                            onClick={() => openEditConfigModal(cfg)}
                            class="p-1.5 text-slate-400 hover:text-white rounded hover:bg-slate-800 transition-colors"
                          >
                            <Edit class="w-3.5 h-3.5" />
                          </button>
                          <button
                            onClick={() => handleDeleteConfig(cfg.id)}
                            class="p-1.5 text-rose-400 hover:text-rose-300 rounded hover:bg-slate-800 transition-colors"
                          >
                            <Trash2 class="w-3.5 h-3.5" />
                          </button>
                        </td>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </div>
          </div>

          {/* Master Layout & Site Plans Gallery */}
          <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 space-y-4">
            <div class="flex items-center justify-between pb-3 border-b border-slate-800">
              <div>
                <h3 class="text-sm font-bold text-white uppercase tracking-wider flex items-center gap-2">
                  <Compass class="w-4 h-4 text-indigo-400" /> Master Layout & Site Plans
                </h3>
                <p class="text-xs text-slate-400 mt-0.5">
                  Project-wide blueprints, cluster plans, and master site layouts
                </p>
              </div>

              <Button variant="secondary" size="sm" onClick={() => setFloorPlanModalOpen(true)}>
                <Upload class="w-3.5 h-3.5" /> Upload Master Plan
              </Button>
            </div>

            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
              <For
                each={project()?.floor_plans}
                fallback={
                  <div class="col-span-full py-8 text-center text-xs text-slate-500 border border-dashed border-slate-800 rounded-xl">
                    No master layout plans uploaded yet.
                  </div>
                }
              >
                {(fp) => (
                  <div class="bg-slate-950 border border-slate-800 rounded-xl overflow-hidden group">
                    <div class="aspect-video bg-slate-900 relative">
                      <img src={fp.thumbnail_url} alt={fp.title} class="w-full h-full object-cover" />
                      <button
                        onClick={() => handleDeleteFloorPlan(fp.id)}
                        class="absolute top-2 right-2 p-1.5 rounded-lg bg-rose-950/80 text-rose-300 hover:bg-rose-900 transition-colors"
                      >
                        <Trash2 class="w-3.5 h-3.5" />
                      </button>
                    </div>
                    <div class="p-3">
                      <div class="text-xs font-semibold text-slate-200 truncate">{fp.title}</div>
                      <a
                        href={fp.url}
                        target="_blank"
                        rel="noopener"
                        class="text-[11px] text-indigo-400 hover:underline mt-1 inline-block"
                      >
                        Open Full Resolution &rarr;
                      </a>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </div>
        </div>
      )}

      {/* --------------------------------------------------------------------- */}
      {/* TAB 4: SPECIFICATIONS                                                 */}
      {/* --------------------------------------------------------------------- */}
      {activeTab() === "specifications" && (
        <div class="space-y-6">
          <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 space-y-6">
            <div class="flex items-center justify-between pb-3 border-b border-slate-800">
              <div>
                <h3 class="text-sm font-bold text-white uppercase tracking-wider flex items-center gap-2">
                  <Sparkles class="w-4 h-4 text-indigo-400" /> Technical Specifications
                </h3>
                <p class="text-xs text-slate-400 mt-0.5">
                  Categorized materials, fittings, structural standards, and fixture highlights
                </p>
              </div>

              <div class="flex items-center gap-2">
                <input
                  type="text"
                  placeholder="New Category (e.g. Flooring)"
                  value={newSpecCategory()}
                  onInput={(e) => setNewSpecCategory(e.currentTarget.value)}
                  class="px-3 py-1.5 bg-slate-950 border border-slate-800 rounded-lg text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
                />
                <Button size="sm" onClick={addSpecCategory}>
                  <Plus class="w-3.5 h-3.5" /> Add Category
                </Button>
              </div>
            </div>

            <div class="space-y-4">
              <For
                each={specs()}
                fallback={
                  <div class="py-12 text-center text-xs text-slate-500 border border-dashed border-slate-800 rounded-xl">
                    No specifications configured. Add categories above to define project finishes.
                  </div>
                }
              >
                {(spec, catIdx) => (
                  <div class="bg-slate-950 border border-slate-800 rounded-xl p-4 space-y-3">
                    <div class="flex items-center justify-between">
                      <h4 class="text-xs font-bold text-indigo-400 uppercase tracking-wider">
                        {spec.category}
                      </h4>
                      <button
                        onClick={() => removeSpecCategory(catIdx())}
                        class="text-rose-400 hover:text-rose-300 text-xs flex items-center gap-1"
                      >
                        <Trash2 class="w-3 h-3" /> Remove Category
                      </button>
                    </div>

                    <div class="space-y-2">
                      <For each={spec.items}>
                        {(item, itemIdx) => (
                          <div class="flex items-center gap-2">
                            <input
                              type="text"
                              value={item}
                              onInput={(e) => updateSpecItem(catIdx(), itemIdx(), e.currentTarget.value)}
                              placeholder="e.g. Premium Italian vitrified tiles in all living areas"
                              class="flex-1 px-3 py-1.5 bg-slate-900 border border-slate-800 rounded-lg text-xs text-white placeholder-slate-600 focus:outline-none focus:border-indigo-500"
                            />
                            <button
                              onClick={() => removeSpecItem(catIdx(), itemIdx())}
                              class="p-1.5 text-slate-500 hover:text-rose-400 transition-colors"
                            >
                              <X class="w-3.5 h-3.5" />
                            </button>
                          </div>
                        )}
                      </For>
                    </div>

                    <button
                      onClick={() => addSpecItem(catIdx())}
                      class="text-xs text-indigo-400 hover:text-indigo-300 font-semibold flex items-center gap-1 pt-1"
                    >
                      <Plus class="w-3 h-3" /> Add Item Line
                    </button>
                  </div>
                )}
              </For>
            </div>
          </div>
        </div>
      )}

      {/* --------------------------------------------------------------------- */}
      {/* TAB 5: LANDMARKS & CONNECTIVITY                                       */}
      {/* --------------------------------------------------------------------- */}
      {activeTab() === "landmarks" && (
        <div class="space-y-6">
          <div class="bg-slate-900 border border-slate-800 rounded-2xl overflow-hidden shadow-sm">
            <div class="flex items-center justify-between px-6 py-4 border-b border-slate-800">
              <div>
                <h3 class="text-sm font-bold text-white uppercase tracking-wider flex items-center gap-2">
                  <Compass class="w-4 h-4 text-indigo-400" /> Connectivity & Nearby Landmarks
                </h3>
                <p class="text-xs text-slate-400 mt-0.5">
                  Neighborhood commute times, transit hubs, schools, hospitals, and commercial parks
                </p>
              </div>

              <Button onClick={() => setLandmarkModalOpen(true)}>
                <Plus class="w-4 h-4" /> Add Landmark
              </Button>
            </div>

            <div class="overflow-x-auto">
              <table class="w-full text-left text-xs">
                <thead class="bg-slate-950/60 text-slate-400 uppercase tracking-wider font-semibold border-b border-slate-800">
                  <tr>
                    <th class="px-6 py-3">Category</th>
                    <th class="px-6 py-3">Landmark Name</th>
                    <th class="px-6 py-3">Distance</th>
                    <th class="px-6 py-3">Duration / Drive Time</th>
                    <th class="px-6 py-3 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-slate-800/60">
                  <For
                    each={landmarks()}
                    fallback={
                      <tr>
                        <td colspan="5" class="px-6 py-12 text-center text-slate-500">
                          No connectivity landmarks added yet.
                        </td>
                      </tr>
                    }
                  >
                    {(lm, idx) => (
                      <tr class="hover:bg-slate-800/40 transition-colors">
                        <td class="px-6 py-4">
                          <Badge variant="default">{lm.category}</Badge>
                        </td>
                        <td class="px-6 py-4 font-bold text-slate-100">{lm.name}</td>
                        <td class="px-6 py-4 text-slate-300">{lm.distance || "—"}</td>
                        <td class="px-6 py-4 text-slate-300">{lm.duration || "—"}</td>
                        <td class="px-6 py-4 text-right">
                          <button
                            onClick={() => removeLandmark(idx())}
                            class="p-1.5 text-rose-400 hover:text-rose-300 rounded hover:bg-slate-800 transition-colors"
                          >
                            <Trash2 class="w-3.5 h-3.5" />
                          </button>
                        </td>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* --------------------------------------------------------------------- */}
      {/* TAB 6: MEDIA GALLERY                                                  */}
      {/* --------------------------------------------------------------------- */}
      {activeTab() === "media" && (
        <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6">
          <MediaManager
            title="Project Media Gallery & Visuals"
            uploadEndpoint={`/api/admin/projects/${projectId()}/media`}
            reorderEndpoint={`/api/admin/projects/${projectId()}/media/reorder`}
            deleteEndpoint={(mid) => `/api/admin/projects/${projectId()}/media/${mid}`}
            media={project()?.media || []}
            onRefresh={fetchProject}
          />
        </div>
      )}

      {/* --------------------------------------------------------------------- */}
      {/* TAB 7: DOCUMENTS VAULT                                                */}
      {/* --------------------------------------------------------------------- */}
      {activeTab() === "documents" && (
        <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 space-y-4">
          <div class="flex items-center justify-between pb-3 border-b border-slate-800">
            <div>
              <h3 class="text-sm font-bold text-white uppercase tracking-wider flex items-center gap-2">
                <FileCheck2 class="w-4 h-4 text-indigo-400" /> Compliance & Legal Documents Vault
              </h3>
              <p class="text-xs text-slate-400 mt-0.5">
                Official brochures, RERA approvals, BU permissions, and deeds
              </p>
            </div>

            <Button size="sm" onClick={() => setDocModalOpen(true)}>
              <Upload class="w-3.5 h-3.5" /> Upload Document
            </Button>
          </div>

          <div class="divide-y divide-slate-800/60">
            <For
              each={project()?.documents}
              fallback={
                <div class="py-12 text-center text-xs text-slate-500 border border-dashed border-slate-800 rounded-xl">
                  No documents in vault yet.
                </div>
              }
            >
              {(doc) => (
                <div class="py-3 flex items-center justify-between gap-4">
                  <div class="flex items-center gap-3 min-w-0">
                    <div class="w-9 h-9 rounded-lg bg-rose-950/60 border border-rose-800/60 text-rose-300 font-bold flex items-center justify-center text-xs shrink-0">
                      PDF
                    </div>
                    <div class="truncate">
                      <div class="text-xs font-semibold text-slate-100 truncate">{doc.file_name}</div>
                      <div class="text-[11px] text-slate-400 uppercase tracking-wider">
                        Type: {doc.doc_type}
                      </div>
                    </div>
                  </div>

                  <div class="flex items-center gap-2">
                    <a
                      href={doc.url}
                      target="_blank"
                      rel="noopener"
                      class="p-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs flex items-center gap-1 font-medium transition-colors"
                    >
                      <ExternalLink class="w-3.5 h-3.5" /> View
                    </a>
                    <button
                      onClick={() => handleDeleteDocument(doc.id)}
                      class="p-2 rounded-lg bg-slate-800 hover:bg-rose-950/60 text-rose-400 transition-colors"
                    >
                      <Trash2 class="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>
              )}
            </For>
          </div>
        </div>
      )}

      {/* --------------------------------------------------------------------- */}
      {/* MODALS                                                                */}
      {/* --------------------------------------------------------------------- */}

      {/* Configuration Modal */}
      <Modal
        isOpen={configModalOpen()}
        onClose={() => setConfigModalOpen(false)}
        title={editingConfigId() ? "Edit Configuration" : "Add Configuration"}
        footer={
          <>
            <Button variant="ghost" onClick={() => setConfigModalOpen(false)}>
              Cancel
            </Button>
            <Button loading={configSubmitting()} onClick={handleSaveConfig}>
              Save Configuration
            </Button>
          </>
        }
      >
        <form onSubmit={handleSaveConfig} class="space-y-4">
          <Input
            label="Configuration Title"
            required
            placeholder="e.g. 3 BHK Luxury Suite"
            value={configName()}
            onInput={(e) => setConfigName(e.currentTarget.value)}
          />

          <div class="grid grid-cols-2 gap-3">
            <Input
              label="BHK"
              type="number"
              step="0.5"
              placeholder="e.g. 3"
              value={configBhk() !== "" ? String(configBhk()) : ""}
              onInput={(e) => setConfigBhk(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
            />
            <Input
              label="Price (₹)"
              type="number"
              placeholder="e.g. 12500000"
              disabled={configPriceOnRequest()}
              value={configPrice() !== "" ? String(configPrice()) : ""}
              onInput={(e) => setConfigPrice(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
            />
          </div>

          <label class="flex items-center gap-2 text-xs text-slate-300 cursor-pointer">
            <input
              type="checkbox"
              checked={configPriceOnRequest()}
              onChange={(e) => setConfigPriceOnRequest(e.currentTarget.checked)}
              class="rounded border-slate-700 bg-slate-800 text-indigo-600 focus:ring-indigo-500 h-3.5 w-3.5"
            />
            <span>Price on Request</span>
          </label>

          <div class="grid grid-cols-3 gap-3">
            <Input
              label="Carpet Area (sq.ft)"
              type="number"
              placeholder="e.g. 1150"
              value={configCarpetArea() !== "" ? String(configCarpetArea()) : ""}
              onInput={(e) => setConfigCarpetArea(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
            />
            <Input
              label="Built-Up Area (sq.ft)"
              type="number"
              placeholder="e.g. 1400"
              value={configBuiltUpArea() !== "" ? String(configBuiltUpArea()) : ""}
              onInput={(e) => setConfigBuiltUpArea(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
            />
            <Input
              label="Super Built-Up (sq.ft)"
              type="number"
              placeholder="e.g. 1750"
              value={configSuperBuiltUpArea() !== "" ? String(configSuperBuiltUpArea()) : ""}
              onInput={(e) => setConfigSuperBuiltUpArea(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
            />
          </div>

          <div>
            <label class="block text-xs font-medium text-slate-300 mb-1">
              Floor Plan Image (1:1 Layout)
            </label>
            <input
              type="file"
              accept="image/*"
              onChange={(e) => setConfigFloorPlanFile(e.currentTarget.files?.[0] || null)}
              class="w-full text-xs text-slate-400 file:mr-3 file:py-1.5 file:px-3 file:rounded-lg file:border-0 file:text-xs file:font-semibold file:bg-slate-800 file:text-slate-200 hover:file:bg-slate-700 cursor-pointer"
            />
          </div>
        </form>
      </Modal>

      {/* Master Floor Plan Modal */}
      <Modal
        isOpen={floorPlanModalOpen()}
        onClose={() => setFloorPlanModalOpen(false)}
        title="Upload Master Site / Layout Plan"
        footer={
          <>
            <Button variant="ghost" onClick={() => setFloorPlanModalOpen(false)}>
              Cancel
            </Button>
            <Button loading={floorPlanSubmitting()} onClick={handleUploadFloorPlan}>
              Upload Plan
            </Button>
          </>
        }
      >
        <form onSubmit={handleUploadFloorPlan} class="space-y-4">
          <Input
            label="Plan Title"
            required
            placeholder="e.g. Master Layout & Phase 1 Cluster Plan"
            value={floorPlanTitle()}
            onInput={(e) => setFloorPlanTitle(e.currentTarget.value)}
          />
          <div>
            <label class="block text-xs font-medium text-slate-300 mb-1">
              Layout Image File
            </label>
            <input
              type="file"
              accept="image/*"
              required
              onChange={(e) => setFloorPlanFile(e.currentTarget.files?.[0] || null)}
              class="w-full text-xs text-slate-400 file:mr-3 file:py-1.5 file:px-3 file:rounded-lg file:border-0 file:text-xs file:font-semibold file:bg-slate-800 file:text-slate-200 hover:file:bg-slate-700 cursor-pointer"
            />
          </div>
        </form>
      </Modal>

      {/* Landmark Modal */}
      <Modal
        isOpen={landmarkModalOpen()}
        onClose={() => setLandmarkModalOpen(false)}
        title="Add Connectivity Landmark"
        footer={
          <>
            <Button variant="ghost" onClick={() => setLandmarkModalOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleAddLandmark}>
              Add Landmark
            </Button>
          </>
        }
      >
        <form onSubmit={handleAddLandmark} class="space-y-4">
          <Select
            label="Category"
            value={lmCategory()}
            onChange={(e) => setLmCategory(e.currentTarget.value)}
            options={[
              { value: "Connectivity", label: "Connectivity & Transit (Metro, Airport, Highway)" },
              { value: "Education", label: "Education (Schools, Colleges)" },
              { value: "Healthcare", label: "Healthcare (Hospitals, Clinics)" },
              { value: "Shopping", label: "Shopping & Retail (Malls, Markets)" },
              { value: "Business", label: "Business Hubs & IT Parks" },
            ]}
          />

          <Input
            label="Landmark Name"
            required
            placeholder="e.g. Thaltej Metro Station"
            value={lmName()}
            onInput={(e) => setLmName(e.currentTarget.value)}
          />

          <div class="grid grid-cols-2 gap-3">
            <Input
              label="Distance"
              placeholder="e.g. 1.8 km"
              value={lmDistance()}
              onInput={(e) => setLmDistance(e.currentTarget.value)}
            />
            <Input
              label="Duration"
              placeholder="e.g. 5 mins drive"
              value={lmDuration()}
              onInput={(e) => setLmDuration(e.currentTarget.value)}
            />
          </div>

          <div class="grid grid-cols-2 gap-3">
            <Input
              label="Latitude (optional)"
              type="number"
              step="0.000001"
              value={lmLat() !== "" ? String(lmLat()) : ""}
              onInput={(e) => setLmLat(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
            />
            <Input
              label="Longitude (optional)"
              type="number"
              step="0.000001"
              value={lmLng() !== "" ? String(lmLng()) : ""}
              onInput={(e) => setLmLng(e.currentTarget.value ? Number(e.currentTarget.value) : "")}
            />
          </div>
        </form>
      </Modal>

      {/* Document Upload Modal */}
      <Modal
        isOpen={docModalOpen()}
        onClose={() => setDocModalOpen(false)}
        title="Upload Compliance Document"
        footer={
          <>
            <Button variant="ghost" onClick={() => setDocModalOpen(false)}>
              Cancel
            </Button>
            <Button loading={docSubmitting()} onClick={handleUploadDocument}>
              Upload Document
            </Button>
          </>
        }
      >
        <form onSubmit={handleUploadDocument} class="space-y-4">
          <Input
            label="Display Name / Document Title"
            required
            placeholder="e.g. Gujarat RERA Certificate"
            value={docDisplayName()}
            onInput={(e) => setDocDisplayName(e.currentTarget.value)}
          />

          <Select
            label="Document Classification"
            value={docType()}
            onChange={(e) => setDocType(e.currentTarget.value)}
            options={[
              { value: "brochure", label: "Official Brochure" },
              { value: "rera", label: "RERA Registration Certificate" },
              { value: "bu_approval", label: "Building Use (BU) Approval" },
              { value: "other", label: "Other Legal Deed / Layout Approval" },
            ]}
          />

          <div>
            <label class="block text-xs font-medium text-slate-300 mb-1">
              Document File (PDF format)
            </label>
            <input
              type="file"
              accept=".pdf"
              required
              onChange={(e) => setDocFile(e.currentTarget.files?.[0] || null)}
              class="w-full text-xs text-slate-400 file:mr-3 file:py-1.5 file:px-3 file:rounded-lg file:border-0 file:text-xs file:font-semibold file:bg-slate-800 file:text-slate-200 hover:file:bg-slate-700 cursor-pointer"
            />
          </div>
        </form>
      </Modal>

      {/* Inline Quick Add Amenity Modal */}
      <Modal
        isOpen={amenityModalOpen()}
        onClose={() => setAmenityModalOpen(false)}
        title="Quick-Add Amenity"
        footer={
          <>
            <Button variant="ghost" onClick={() => setAmenityModalOpen(false)}>
              Cancel
            </Button>
            <Button loading={quickAmenityLoading()} onClick={handleQuickAddAmenity}>
              Add & Link Amenity
            </Button>
          </>
        }
      >
        <div class="space-y-4">
          <Input
            label="Amenity Title"
            required
            placeholder="e.g. Squash Court, EV Charging Station"
            value={quickAmenityName()}
            onInput={(e) => setQuickAmenityName(e.currentTarget.value)}
          />
        </div>
      </Modal>
    </div>
  );
}

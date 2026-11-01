import { createSignal, onMount, Show, For } from "solid-js";
import { useParams, useNavigate, useSearchParams, A } from "@solidjs/router";
import {
  MapPin,
  Calendar,
  FileText,
  Plus,
  Trash2,
  ExternalLink,
  ArrowLeft,
  Upload,
  Edit,
} from "lucide-solid";

import { Button } from "@/shared/components/Button";
import { Badge } from "@/shared/components/Badge";
import { Modal } from "@/shared/components/Modal";
import { Input } from "@/shared/components/Input";
import { Select } from "@/shared/components/Select";
import { Textarea } from "@/shared/components/Textarea";
import { MediaManager } from "@/apps/admin/components/MediaManager";
import { api } from "@/shared/api/client";
import { debounce } from "@/shared/utils/debounce";
import {
  ProjectDetailData,
  PropertyGlobalItem,
  MetaOptionsResponse,
  PropertyFormPayload,
  ProjectFormPayload,
} from "@/shared/api/types";


export function ProjectDetail() {
  const params = useParams();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const projectId = () => Number(params.id);

  const [project, setProject] = createSignal<ProjectDetailData | null>(null);
  const [properties, setProperties] = createSignal<PropertyGlobalItem[]>([]);
  const [meta, setMeta] = createSignal<MetaOptionsResponse | null>(null);
  const [loading, setLoading] = createSignal(true);

  const activeTab = (): "overview" | "units" | "media" | "documents" => {
    const rawTab = searchParams.tab;
    const tab = (Array.isArray(rawTab) ? rawTab[0] : rawTab)?.toLowerCase();
    if (tab === "units" || tab === "inventory") return "units";
    if (tab === "media" || tab === "gallery") return "media";
    if (tab === "documents" || tab === "docs") return "documents";
    return "overview";
  };

  const selectTab = (tab: "overview" | "units" | "media" | "documents") => {
    setSearchParams({ tab });
  };

  // Unit helpers
  const getStatusOptions = (listingType: string) => {
    if (listingType === "rent") {
      return [
        { value: "active", label: "Available for Rent" },
        { value: "closed", label: "Rented" },
        { value: "withdrawn", label: "Reserved / Withdrawn" },
      ];
    }
    if (listingType === "lease") {
      return [
        { value: "active", label: "Available for Lease" },
        { value: "closed", label: "Leased" },
        { value: "withdrawn", label: "Reserved / Withdrawn" },
      ];
    }
    return [
      { value: "active", label: "Available for Sale" },
      { value: "sold", label: "Sold" },
      { value: "withdrawn", label: "Reserved / Withdrawn" },
      { value: "closed", label: "Closed" },
    ];
  };

  const formatListingStatus = (status?: string | null, listingType?: string | null) => {
    const s = (status || "active").toLowerCase();
    const lt = (listingType || "sale").toLowerCase();
    if (s === "active" || s === "available") {
      return lt === "rent" ? "Available (Rent)" : lt === "lease" ? "Available (Lease)" : "Available";
    }
    if (s === "sold") return "Sold";
    if (s === "closed") {
      return lt === "rent" ? "Rented" : lt === "lease" ? "Leased" : "Closed";
    }
    if (s === "withdrawn") return "Reserved / Withdrawn";
    return status || "Active";
  };

  // Unit filter state
  const [unitFilterSearch, setUnitFilterSearch] = createSignal("");
  const [debouncedUnitFilterSearch, setDebouncedUnitFilterSearch] = createSignal("");
  const updateDebouncedUnitSearch = debounce((v: string) => {
    setDebouncedUnitFilterSearch(v);
  }, 250);
  const [unitFilterStatus, setUnitFilterStatus] = createSignal("all");

  const filteredUnits = () => {
    return properties().filter((p) => {
      const matchSearch =
        !debouncedUnitFilterSearch().trim() ||
        p.unit_title.toLowerCase().includes(debouncedUnitFilterSearch().toLowerCase()) ||
        p.type_name.toLowerCase().includes(debouncedUnitFilterSearch().toLowerCase());
      const pStatus = p.status ? p.status.toLowerCase() : "active";
      const filter = unitFilterStatus().toLowerCase();
      const matchStatus =
        filter === "all" ||
        (filter === "active" && (pStatus === "active" || pStatus === "available")) ||
        pStatus === filter;
      return matchSearch && matchStatus;
    });
  };

  // Add Unit Modal state
  const [addUnitModalOpen, setAddUnitModalOpen] = createSignal(false);
  const [unitSubmitting, setUnitSubmitting] = createSignal(false);
  const [unitError, setUnitError] = createSignal<string | null>(null);
  const [unitTypeId, setUnitTypeId] = createSignal<number | "">("");
  const [unitNumber, setUnitNumber] = createSignal("");
  const [unitBuilding, setUnitBuilding] = createSignal("");
  const [unitFloor, setUnitFloor] = createSignal<number | "">("");
  const [unitPrice, setUnitPrice] = createSignal<number | "">("");
  const [unitBuiltUpArea, setUnitBuiltUpArea] = createSignal<number | "">("");
  const [unitBedrooms, setUnitBedrooms] = createSignal<number | "">("");
  const [unitBathrooms, setUnitBathrooms] = createSignal<number | "">("");
  const [unitListingType, setUnitListingType] = createSignal("sale");
  const [unitStatus, setUnitStatus] = createSignal("active");

  // Document Upload Modal state

  const [uploadDocModalOpen, setUploadDocModalOpen] = createSignal(false);
  const [docUploading, setDocUploading] = createSignal(false);
  const [docType, setDocType] = createSignal("brochure");
  const [docDisplayName, setDocDisplayName] = createSignal("");
  const [docFile, setDocFile] = createSignal<File | null>(null);
  const [docError, setDocError] = createSignal<string | null>(null);

  // Edit Project Modal state
  const [editProjectModalOpen, setEditProjectModalOpen] = createSignal(false);
  const [editProjectSubmitting, setEditProjectSubmitting] = createSignal(false);
  const [editProjectError, setEditProjectError] = createSignal<string | null>(null);
  const [editProjectName, setEditProjectName] = createSignal("");
  const [editProjectCategory, setEditProjectCategory] = createSignal("RESIDENTIAL");
  const [editProjectOwnerId, setEditProjectOwnerId] = createSignal<number | "">("");
  const [editProjectLocationId, setEditProjectLocationId] = createSignal<number | "">("");
  const [editProjectDescription, setEditProjectDescription] = createSignal("");
  const [editProjectStartDate, setEditProjectStartDate] = createSignal("");
  const [editProjectLaunchDate, setEditProjectLaunchDate] = createSignal("");
  const [editProjectPossessionDate, setEditProjectPossessionDate] = createSignal("");
  const [editProjectAmenities, setEditProjectAmenities] = createSignal<number[]>([]);

  const fetchProjectData = async (silent = false) => {
    if (!silent && !project()) {
      setLoading(true);
    }
    try {
      const [projData, propsData] = await Promise.all([
        api.get<ProjectDetailData>(`/api/admin/projects/${projectId()}`),
        api.get<PropertyGlobalItem[]>(`/api/admin/projects/${projectId()}/properties`),
      ]);
      setProject(projData);
      setProperties(propsData);
    } catch (e: any) {
      alert(e.message || "Failed to load project details");
      navigate("/projects");
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
    fetchProjectData();
  });

  const availablePropertyTypes = () => {
    if (!meta()?.property_types) return [];
    const projCat = project()?.category?.toUpperCase();
    const matched = meta()!.property_types.filter(
      (pt) => !projCat || pt.category.toUpperCase() === projCat
    );
    return (matched.length > 0 ? matched : meta()!.property_types).map((pt) => ({
      value: pt.id,
      label: `${pt.name} (${pt.category})`,
    }));
  };

  const openEditProjectModal = () => {
    const p = project();
    if (!p) return;
    setEditProjectName(p.name);
    setEditProjectCategory(p.category.toUpperCase());
    setEditProjectOwnerId(p.owner_id);
    setEditProjectLocationId(p.location_id);
    setEditProjectDescription(p.description || "");
    setEditProjectStartDate(p.start_date || "");
    setEditProjectLaunchDate(p.launch_date || "");
    setEditProjectPossessionDate(p.possession_date || "");
    setEditProjectAmenities(p.amenity_ids ? [...p.amenity_ids] : []);
    setEditProjectError(null);
    setEditProjectModalOpen(true);
  };

  const toggleAmenity = (id: number) => {
    setEditProjectAmenities((prev) =>
      prev.includes(id) ? prev.filter((a) => a !== id) : [...prev, id]
    );
  };

  const handleUpdateProject = async (e: Event) => {
    e.preventDefault();
    if (!editProjectName().trim()) {
      setEditProjectError("Project name is required");
      return;
    }
    if (!editProjectOwnerId()) {
      setEditProjectError("Please select a developer/owner");
      return;
    }
    if (!editProjectLocationId()) {
      setEditProjectError("Please select a location");
      return;
    }

    setEditProjectSubmitting(true);
    setEditProjectError(null);

    const payload: ProjectFormPayload = {
      name: editProjectName().trim(),
      category: editProjectCategory(),
      project_owner_id: Number(editProjectOwnerId()),
      location_id: Number(editProjectLocationId()),
      description: editProjectDescription().trim() || null,
      start_date: editProjectStartDate() || null,
      launch_date: editProjectLaunchDate() || null,
      possession_date: editProjectPossessionDate() || null,
      amenities: editProjectAmenities(),
    };

    try {
      await api.put(`/api/admin/projects/${projectId()}`, payload);
      setEditProjectModalOpen(false);
      fetchProjectData(true);
    } catch (err: any) {
      setEditProjectError(err.message || "Failed to update project");
    } finally {
      setEditProjectSubmitting(false);
    }
  };

  const handleCreateUnit = async (e: Event) => {
    e.preventDefault();
    if (!unitTypeId() || unitPrice() === "") {
      setUnitError("Please select property type and price.");
      return;
    }

    setUnitSubmitting(true);
    setUnitError(null);

    const payload: PropertyFormPayload = {
      property_type_id: Number(unitTypeId()),
      unit_number: unitNumber().trim() || undefined,
      building: unitBuilding().trim() || undefined,
      floor_number: unitFloor() !== "" ? Number(unitFloor()) : undefined,
      price: Number(unitPrice()),
      built_up_area: unitBuiltUpArea() !== "" ? Number(unitBuiltUpArea()) : undefined,
      bedroom_count: unitBedrooms() !== "" ? Number(unitBedrooms()) : undefined,
      bathroom_count: unitBathrooms() !== "" ? Number(unitBathrooms()) : undefined,
      status: unitStatus(),
      listing_type: unitListingType(),
      amenities: [],
    };

    try {
      await api.post(`/api/admin/projects/${projectId()}/properties`, payload);
      setAddUnitModalOpen(false);
      resetUnitForm();
      fetchProjectData(true);
    } catch (err: any) {
      setUnitError(err.message || "Failed to create unit");
    } finally {
      setUnitSubmitting(false);
    }
  };

  const resetUnitForm = () => {
    setUnitTypeId("");
    setUnitNumber("");
    setUnitBuilding("");
    setUnitFloor("");
    setUnitPrice("");
    setUnitBuiltUpArea("");
    setUnitBedrooms("");
    setUnitBathrooms("");
    setUnitListingType("sale");
    setUnitStatus("active");
    setUnitError(null);
  };


  const handleDeleteProject = async () => {
    if (!confirm("Are you sure you want to delete this project? All associated media and records will be removed."))
      return;
    try {
      await api.delete(`/api/admin/projects/${projectId()}`);
      navigate("/projects");
    } catch (e: any) {
      alert(e.message || "Failed to delete project");
    }
  };

  const openUploadDocModal = () => {
    setDocError(null);
    setDocFile(null);
    setDocDisplayName("");
    setDocType("brochure");
    setUploadDocModalOpen(true);
  };

  const handleUploadDocumentSubmit = async (e: Event) => {
    e.preventDefault();
    if (!docFile()) {
      setDocError("Please select a document file to upload.");
      return;
    }
    setDocUploading(true);
    setDocError(null);
    const formData = new FormData();
    formData.append("file", docFile()!);
    formData.append("doc_type", docType());
    if (docDisplayName().trim()) {
      formData.append("display_name", docDisplayName().trim());
    }

    try {
      await api.upload(`/api/admin/projects/${projectId()}/documents`, formData);
      setUploadDocModalOpen(false);
      setDocFile(null);
      setDocDisplayName("");
      setDocType("brochure");
      fetchProjectData(true);
    } catch (e: any) {
      setDocError(e.message || "Failed to upload document");
    } finally {
      setDocUploading(false);
    }
  };

  const handleDeleteDocument = async (docId: string) => {
    if (!confirm("Delete this document?")) return;
    try {
      await api.delete(`/api/admin/projects/${projectId()}/documents/${docId}`);
      fetchProjectData(true);
    } catch (e: any) {
      alert(e.message || "Failed to delete document");
    }
  };


  return (
    <div class="space-y-6">
      {/* Top back navigation and Actions */}
      <div class="flex items-center justify-between">
        <A
          href="/projects"
          class="inline-flex items-center gap-1.5 text-xs font-medium text-slate-400 hover:text-white transition-colors"
        >
          <ArrowLeft class="w-3.5 h-3.5" /> Back to Projects Directory
        </A>

        <div class="flex items-center gap-2">
          <Button variant="secondary" size="sm" onClick={openEditProjectModal}>
            <Edit class="w-3.5 h-3.5" /> Edit Project
          </Button>
          <Button variant="danger" size="sm" onClick={handleDeleteProject}>
            <Trash2 class="w-3.5 h-3.5" /> Delete Project
          </Button>
        </div>

      </div>

      <Show when={!loading() && project()}>
        {(proj) => (
          <>
            {/* Project Header Banner */}
            <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 sm:p-8">
              <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div>
                  <div class="flex items-center gap-3">
                    <h1 class="text-2xl font-bold text-white">{proj().name}</h1>
                    <Badge
                      variant={
                        proj().category === "RESIDENTIAL"
                          ? "residential"
                          : proj().category === "COMMERCIAL"
                          ? "commercial"
                          : "land"
                      }
                    >
                      {proj().category}
                    </Badge>
                  </div>
                  <p class="text-xs text-slate-400 mt-1">
                    Developed by <span class="text-slate-200 font-semibold">{proj().owner_name}</span>
                  </p>

                  <div class="flex flex-wrap items-center gap-4 text-xs text-slate-400 mt-3">
                    <span class="inline-flex items-center gap-1">
                      <MapPin class="w-3.5 h-3.5 text-slate-500" /> {proj().location_name}
                    </span>
                    {proj().launch_date && (
                      <span class="inline-flex items-center gap-1">
                        <Calendar class="w-3.5 h-3.5 text-slate-500" /> Launched: {proj().launch_date}
                      </span>
                    )}
                    {proj().possession_date && (
                      <span class="inline-flex items-center gap-1">
                        <Calendar class="w-3.5 h-3.5 text-slate-500" /> Possession: {proj().possession_date}
                      </span>
                    )}
                  </div>
                </div>

                <div class="flex items-center gap-3">
                  <div class="text-right">
                    <div class="text-2xl font-bold font-mono text-white">
                      {proj().property_summary.total}
                    </div>
                    <div class="text-[11px] text-slate-500 uppercase tracking-wider font-medium">
                      Total Units
                    </div>
                  </div>
                  <div class="text-right pl-4 border-l border-slate-800">
                    <div class="text-2xl font-bold font-mono text-emerald-400">
                      {proj().property_summary.available}
                    </div>
                    <div class="text-[11px] text-slate-500 uppercase tracking-wider font-medium">
                      Available
                    </div>
                  </div>
                </div>
              </div>

              {/* Navigation Tabs */}
              <div class="flex items-center gap-2 mt-8 pt-4 border-t border-slate-800/80 overflow-x-auto">
                <button
                  onClick={() => selectTab("overview")}
                  class={`px-4 py-2 text-xs font-semibold rounded-lg transition-colors cursor-pointer ${
                    activeTab() === "overview"
                      ? "bg-indigo-600 text-white"
                      : "text-slate-400 hover:text-white hover:bg-slate-800"
                  }`}
                >
                  Overview
                </button>
                <button
                  onClick={() => selectTab("units")}
                  class={`px-4 py-2 text-xs font-semibold rounded-lg transition-colors cursor-pointer ${
                    activeTab() === "units"
                      ? "bg-indigo-600 text-white"
                      : "text-slate-400 hover:text-white hover:bg-slate-800"
                  }`}
                >
                  Units & Inventory ({properties().length})
                </button>
                <button
                  onClick={() => selectTab("media")}
                  class={`px-4 py-2 text-xs font-semibold rounded-lg transition-colors cursor-pointer ${
                    activeTab() === "media"
                      ? "bg-indigo-600 text-white"
                      : "text-slate-400 hover:text-white hover:bg-slate-800"
                  }`}
                >
                  Media Gallery ({proj().media.length})
                </button>
                <button
                  onClick={() => selectTab("documents")}
                  class={`px-4 py-2 text-xs font-semibold rounded-lg transition-colors cursor-pointer ${
                    activeTab() === "documents"
                      ? "bg-indigo-600 text-white"
                      : "text-slate-400 hover:text-white hover:bg-slate-800"
                  }`}
                >
                  Brochures & Docs ({proj().documents.length})
                </button>
              </div>
            </div>

            {/* Tab: Overview */}
            <Show when={activeTab() === "overview"}>
              <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div class="md:col-span-2 space-y-6">
                  {/* Description Box */}
                  <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6">
                    <h3 class="text-sm font-semibold text-white mb-3">Project Description</h3>
                    <p class="text-sm text-slate-300 leading-relaxed whitespace-pre-line">
                      {proj().description || "No description provided for this development project."}
                    </p>
                  </div>

                  {/* Amenities */}
                  <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6">
                    <h3 class="text-sm font-semibold text-white mb-3">Linked Amenities</h3>
                    <div class="flex flex-wrap gap-2">
                      <For each={proj().amenities}>
                        {(amenity) => (
                          <span class="px-3 py-1 bg-slate-800 border border-slate-700/80 rounded-lg text-xs text-slate-200">
                            {amenity}
                          </span>
                        )}
                      </For>
                      {proj().amenities.length === 0 && (
                        <p class="text-xs text-slate-500">No amenities linked.</p>
                      )}
                    </div>
                  </div>
                </div>

                {/* Sidebar breakdown */}
                <div class="space-y-6">
                  <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 space-y-4">
                    <h3 class="text-sm font-semibold text-white">Inventory Breakdown</h3>
                    <div class="space-y-2 text-xs">
                      <div class="flex justify-between py-1.5 border-b border-slate-800">
                        <span class="text-slate-400">Residential</span>
                        <span class="font-mono text-slate-100 font-bold">{proj().property_summary.residential}</span>
                      </div>
                      <div class="flex justify-between py-1.5 border-b border-slate-800">
                        <span class="text-slate-400">Commercial</span>
                        <span class="font-mono text-slate-100 font-bold">{proj().property_summary.commercial}</span>
                      </div>
                      <div class="flex justify-between py-1.5 border-b border-slate-800">
                        <span class="text-slate-400">Plots / Land</span>
                        <span class="font-mono text-slate-100 font-bold">{proj().property_summary.land}</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </Show>

            {/* Tab: Units */}
            <Show when={activeTab() === "units"}>
              <div class="bg-slate-900 border border-slate-800 rounded-2xl overflow-hidden shadow-sm space-y-4 p-6">
                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3 border-b border-slate-800">
                  <div class="flex items-center gap-2 flex-1 max-w-md">
                    <Input
                      placeholder="Filter by unit, building, or type..."
                      value={unitFilterSearch()}
                      onInput={(e) => {
                        setUnitFilterSearch(e.currentTarget.value);
                        updateDebouncedUnitSearch(e.currentTarget.value);
                      }}
                    />
                    <div class="w-36 shrink-0">
                      <Select
                        value={unitFilterStatus()}
                        onChange={(e) => setUnitFilterStatus(e.currentTarget.value)}
                        options={[
                          { value: "all", label: "All Status" },
                          { value: "active", label: "Available" },
                          { value: "sold", label: "Sold" },
                          { value: "closed", label: "Closed / Rented" },
                          { value: "withdrawn", label: "Reserved / Withdrawn" },
                        ]}
                      />
                    </div>
                  </div>

                  <Button size="sm" onClick={() => setAddUnitModalOpen(true)}>
                    <Plus class="w-3.5 h-3.5" /> Add Unit
                  </Button>
                </div>

                <div class="overflow-x-auto">
                  <table class="w-full text-left text-xs">
                    <thead class="bg-slate-950 text-slate-400 uppercase tracking-wider font-semibold border-b border-slate-800">
                      <tr>
                        <th class="px-4 py-3">Unit / Building</th>
                        <th class="px-4 py-3">Category</th>
                        <th class="px-4 py-3">Type</th>
                        <th class="px-4 py-3">Price</th>
                        <th class="px-4 py-3">Status</th>
                        <th class="px-4 py-3 text-right">Action</th>
                      </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-800/60">
                      <For each={filteredUnits()}>
                        {(p) => (
                          <tr class="hover:bg-slate-800/40 transition-colors">
                            <td class="px-4 py-3 font-semibold text-slate-100">
                              <A href={`/properties/${p.id}`} class="hover:text-indigo-400">
                                {p.unit_title}
                              </A>
                            </td>
                            <td class="px-4 py-3">
                              <Badge variant="default">{p.category}</Badge>
                            </td>
                            <td class="px-4 py-3 text-slate-300">{p.type_name}</td>
                            <td class="px-4 py-3 font-mono font-semibold text-slate-100">
                              ${p.price.toLocaleString()}
                            </td>
                            <td class="px-4 py-3">
                              <Badge
                                variant={
                                  p.status === "active" || p.status === "available"
                                    ? "success"
                                    : p.status === "sold"
                                    ? "danger"
                                    : "warning"
                                }
                              >
                                {formatListingStatus(p.status)}
                              </Badge>
                            </td>

                            <td class="px-4 py-3 text-right">
                              <A href={`/properties/${p.id}`} class="text-indigo-400 hover:text-indigo-300 font-medium">
                                Edit Unit →
                              </A>
                            </td>
                          </tr>
                        )}
                      </For>
                      {filteredUnits().length === 0 && (
                        <tr>
                          <td colspan="6" class="px-4 py-10 text-center text-slate-500">
                            {properties().length === 0
                              ? "No units created in this project yet."
                              : "No units match your filter criteria."}
                          </td>
                        </tr>
                      )}
                    </tbody>
                  </table>
                </div>
              </div>
            </Show>

            {/* Tab: Media Gallery */}
            <Show when={activeTab() === "media"}>
              <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6">
                <h3 class="text-base font-bold text-white mb-4">Project Media Gallery</h3>
                <MediaManager
                  uploadEndpoint={`/api/admin/projects/${projectId()}/media`}
                  reorderEndpoint={`/api/admin/projects/${projectId()}/media/reorder`}
                  deleteEndpoint={(id) => `/api/admin/projects/${projectId()}/media/${id}`}
                  media={proj().media}
                  onRefresh={() => fetchProjectData(true)}
                />
              </div>
            </Show>

            {/* Tab: Documents */}
            <Show when={activeTab() === "documents"}>
              <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 space-y-6">
                <div class="flex items-center justify-between">
                  <div>
                    <h3 class="text-base font-bold text-white">Project Documents</h3>
                    <p class="text-xs text-slate-400">Brochures, RERA certificates, approvals, and legal documents</p>
                  </div>
                  <div>
                    <Button
                      size="sm"
                      onClick={openUploadDocModal}
                    >
                      <Upload class="w-3.5 h-3.5" /> Upload Document
                    </Button>
                  </div>
                </div>

                <div class="divide-y divide-slate-800 border border-slate-800 rounded-xl overflow-hidden bg-slate-950">
                  <For each={proj().documents}>
                    {(doc) => (
                      <div class="flex items-center justify-between p-4 hover:bg-slate-900/50 transition-colors">
                        <div class="flex items-center gap-3">
                          <FileText class="w-5 h-5 text-indigo-400 shrink-0" />
                          <div>
                            <div class="flex items-center gap-2">
                              <span class="px-2 py-0.5 rounded text-[10px] font-mono uppercase font-semibold bg-indigo-950/60 text-indigo-400 border border-indigo-800/60">
                                {doc.doc_type || "brochure"}
                              </span>
                              <a
                                href={doc.url}
                                target="_blank"
                                rel="noreferrer"
                                class="text-sm font-semibold text-slate-200 hover:text-indigo-400 transition-colors flex items-center gap-1.5"
                              >
                                {doc.file_name} <ExternalLink class="w-3 h-3 text-slate-500" />
                              </a>
                            </div>
                            {doc.file_size && (
                              <p class="text-[11px] text-slate-500 font-mono mt-0.5">
                                {(doc.file_size / 1024 / 1024).toFixed(2)} MB
                              </p>
                            )}
                          </div>
                        </div>

                        <Button
                          variant="ghost"
                          size="sm"
                          class="text-rose-400 hover:text-rose-300"
                          onClick={() => handleDeleteDocument(doc.id)}
                        >
                          <Trash2 class="w-3.5 h-3.5" />
                        </Button>
                      </div>
                    )}
                  </For>
                  {proj().documents.length === 0 && (
                    <div class="p-8 text-center text-xs text-slate-500">
                      No documents or brochures uploaded yet.
                    </div>
                  )}
                </div>
              </div>
            </Show>

          </>
        )}
      </Show>

      {/* Add Unit Modal */}
      <Modal
        isOpen={addUnitModalOpen()}
        onClose={() => setAddUnitModalOpen(false)}
        title="Add Unit to Project"
        description="Allocate an apartment, office, or plot to this development."
        footer={
          <>
            <Button variant="ghost" onClick={() => setAddUnitModalOpen(false)}>
              Cancel
            </Button>
            <Button loading={unitSubmitting()} onClick={handleCreateUnit}>
              Save Unit
            </Button>
          </>
        }
      >
        <form onSubmit={handleCreateUnit} class="space-y-4">
          {unitError() && (
            <div class="p-3 bg-rose-950/60 border border-rose-800 rounded-lg text-xs text-rose-300">
              {unitError()}
            </div>
          )}

          <Select
            label="Property Type"
            required
            placeholder="Select unit type"
            value={unitTypeId()}
            onChange={(e) => setUnitTypeId(Number(e.currentTarget.value))}
            options={availablePropertyTypes()}
          />

          <div class="grid grid-cols-2 gap-4">
            <Input
              label="Unit / Flat Number"
              placeholder="e.g. 402B"
              value={unitNumber()}
              onInput={(e) => setUnitNumber(e.currentTarget.value)}
            />
            <Input
              label="Tower / Building"
              placeholder="e.g. Tower B"
              value={unitBuilding()}
              onInput={(e) => setUnitBuilding(e.currentTarget.value)}
            />
          </div>

          <div class="grid grid-cols-2 gap-4">
            <Input
              label="Floor Number"
              type="number"
              placeholder="e.g. 4"
              value={unitFloor()}
              onInput={(e) => setUnitFloor(Number(e.currentTarget.value))}
            />
            <Input
              label="Price ($)"
              type="number"
              required
              placeholder="e.g. 350000"
              value={unitPrice()}
              onInput={(e) => setUnitPrice(Number(e.currentTarget.value))}
            />
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <Select
              label="Listing Type"
              value={unitListingType()}
              onChange={(e) => {
                const lt = e.currentTarget.value;
                setUnitListingType(lt);
                setUnitStatus("active");
              }}
              options={[
                { value: "sale", label: "For Sale" },
                { value: "rent", label: "For Rent" },
                { value: "lease", label: "For Lease" },
              ]}
            />
            <Select
              label="Status"
              value={unitStatus()}
              onChange={(e) => setUnitStatus(e.currentTarget.value)}
              options={getStatusOptions(unitListingType())}
            />
            <Input
              label="Built-Up Area (sq ft)"
              type="number"
              placeholder="e.g. 1450"
              value={unitBuiltUpArea()}
              onInput={(e) => setUnitBuiltUpArea(Number(e.currentTarget.value))}
            />
          </div>


          {project()?.category === "RESIDENTIAL" && (
            <div class="grid grid-cols-2 gap-4 pt-2 border-t border-slate-800">
              <Input
                label="Bedrooms"
                type="number"
                placeholder="e.g. 3"
                value={unitBedrooms()}
                onInput={(e) => setUnitBedrooms(Number(e.currentTarget.value))}
              />
              <Input
                label="Bathrooms"
                type="number"
                placeholder="e.g. 2"
                value={unitBathrooms()}
                onInput={(e) => setUnitBathrooms(Number(e.currentTarget.value))}
              />
            </div>
          )}
        </form>
      </Modal>

      {/* Edit Project Modal */}
      <Modal
        isOpen={editProjectModalOpen()}
        onClose={() => setEditProjectModalOpen(false)}
        title="Edit Project Details"
        description="Update development name, developer assignment, category, timeline, or amenities."
        footer={
          <>
            <Button variant="ghost" onClick={() => setEditProjectModalOpen(false)}>
              Cancel
            </Button>
            <Button loading={editProjectSubmitting()} onClick={handleUpdateProject}>
              Save Changes
            </Button>
          </>
        }
      >
        <form onSubmit={handleUpdateProject} class="space-y-4">
          {editProjectError() && (
            <div class="p-3 bg-rose-950/60 border border-rose-800 rounded-lg text-xs text-rose-300">
              {editProjectError()}
            </div>
          )}

          <Input
            label="Project Name"
            required
            placeholder="e.g. Skyline Towers"
            value={editProjectName()}
            onInput={(e) => setEditProjectName(e.currentTarget.value)}
          />

          <div class="grid grid-cols-2 gap-4">
            <Select
              label="Category"
              required
              value={editProjectCategory()}
              onChange={(e) => setEditProjectCategory(e.currentTarget.value)}
              options={[
                { value: "RESIDENTIAL", label: "Residential" },
                { value: "COMMERCIAL", label: "Commercial" },
                { value: "LAND", label: "Plots / Land" },
              ]}
            />
            <Select
              label="Developer / Owner"
              required
              placeholder="Select developer"
              value={editProjectOwnerId()}
              onChange={(e) => setEditProjectOwnerId(Number(e.currentTarget.value))}
              options={meta()?.owners.map((o) => ({ value: o.id, label: o.name })) || []}
            />
          </div>

          <Select
            label="Location / Region"
            required
            placeholder="Select location"
            value={editProjectLocationId()}
            onChange={(e) => setEditProjectLocationId(Number(e.currentTarget.value))}
            options={
              meta()?.locations.map((l) => ({
                value: l.id,
                label: l.formatted_address,
              })) || []
            }

          />

          <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <Input
              label="Start Date"
              type="date"
              value={editProjectStartDate()}
              onInput={(e) => setEditProjectStartDate(e.currentTarget.value)}
            />
            <Input
              label="Launch Date"
              type="date"
              value={editProjectLaunchDate()}
              onInput={(e) => setEditProjectLaunchDate(e.currentTarget.value)}
            />
            <Input
              label="Possession Date"
              type="date"
              value={editProjectPossessionDate()}
              onInput={(e) => setEditProjectPossessionDate(e.currentTarget.value)}
            />
          </div>

          <Textarea
            label="Project Description"
            placeholder="Provide architectural highlights, location advantages, or master development overview..."
            rows={3}
            value={editProjectDescription()}
            onInput={(e) => setEditProjectDescription(e.currentTarget.value)}
          />

          <div>
            <label class="block text-xs font-medium text-slate-300 mb-1.5">
              Associated Amenities
            </label>
            <div class="grid grid-cols-2 sm:grid-cols-3 gap-2 max-h-48 overflow-y-auto p-2 bg-slate-950 border border-slate-800 rounded-lg">
              <For each={meta()?.amenities || []}>
                {(amenity) => {
                  const isChecked = () => editProjectAmenities().includes(amenity.id);
                  return (
                    <label class="flex items-center gap-2 p-1.5 rounded hover:bg-slate-900 cursor-pointer text-xs text-slate-300 select-none">
                      <input
                        type="checkbox"
                        checked={isChecked()}
                        onChange={() => toggleAmenity(amenity.id)}
                        class="rounded border-slate-700 text-indigo-600 focus:ring-indigo-500/20 bg-slate-900"
                      />
                      <span class="truncate">{amenity.name}</span>
                    </label>
                  );
                }}
              </For>
            </div>
          </div>
        </form>
      </Modal>

      {/* Upload Document Modal */}
      <Modal
        isOpen={uploadDocModalOpen()}
        onClose={() => setUploadDocModalOpen(false)}
        title="Upload Project Document"
        description="Attach brochures, approvals, or legal documentation to this development."
        footer={
          <>
            <Button variant="ghost" onClick={() => setUploadDocModalOpen(false)}>
              Cancel
            </Button>
            <Button loading={docUploading()} onClick={handleUploadDocumentSubmit}>
              Upload Document
            </Button>
          </>
        }
      >
        <form onSubmit={handleUploadDocumentSubmit} class="space-y-4">
          {docError() && (
            <div class="p-3 bg-rose-950/60 border border-rose-800 rounded-lg text-xs text-rose-300">
              {docError()}
            </div>
          )}

          <Select
            label="Document Classification"
            required
            value={docType()}
            onChange={(e) => setDocType(e.currentTarget.value)}
            options={[
              { value: "brochure", label: "Marketing Brochure" },
              { value: "rera", label: "RERA Registration Certificate" },
              { value: "bu_approval", label: "BU Permission / Building Approval" },
              { value: "other", label: "Other Legal / Technical Document" },
            ]}
          />

          <Input
            label="Display Title (optional)"
            placeholder="e.g. Master Layout & Phase 1 Brochure"
            value={docDisplayName()}
            onInput={(e) => setDocDisplayName(e.currentTarget.value)}
          />

          <div>
            <label class="block text-xs font-medium text-slate-300 mb-1.5">
              Document File (PDF, DOC, DOCX) <span class="text-rose-400">*</span>
            </label>
            <input
              type="file"
              accept=".pdf,.doc,.docx"
              required
              onChange={(e) => setDocFile(e.currentTarget.files?.[0] || null)}
              class="w-full text-xs text-slate-400 file:mr-3 file:py-2 file:px-4 file:rounded-lg file:border-0 file:text-xs file:font-semibold file:bg-slate-800 file:text-slate-200 hover:file:bg-slate-700 cursor-pointer"
            />
          </div>
        </form>
      </Modal>

    </div>
  );
}

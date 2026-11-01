import { createSignal, onMount, For, Show } from "solid-js";
import { A, useSearchParams } from "@solidjs/router";
import { Plus, ExternalLink } from "lucide-solid";
import { Button } from "@/shared/components/Button";
import { Input } from "@/shared/components/Input";
import { Select } from "@/shared/components/Select";
import { Badge } from "@/shared/components/Badge";
import { Modal } from "@/shared/components/Modal";
import { api } from "@/shared/api/client";
import {
  PropertyGlobalItem,
  MetaOptionsResponse,
  PropertyFormPayload,
} from "@/shared/api/types";
import { debounce } from "@/shared/utils/debounce";

export function PropertiesList() {
  const [params] = useSearchParams();
  const [properties, setProperties] = createSignal<PropertyGlobalItem[]>([]);
  const [meta, setMeta] = createSignal<MetaOptionsResponse | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [search, setSearch] = createSignal("");
  const [selectedCategory, setSelectedCategory] = createSignal("");
  const [selectedProjectId, setSelectedProjectId] = createSignal<number | "">("");

  const debouncedFetchProperties = debounce(() => {
    fetchProperties();
  }, 300);

  // Create Unit Modal state
  const [modalOpen, setModalOpen] = createSignal(params.action === "new");
  const [submitting, setSubmitting] = createSignal(false);
  const [formError, setFormError] = createSignal<string | null>(null);

  const [formProjectId, setFormProjectId] = createSignal<number | "">("");
  const [formTypeId, setFormTypeId] = createSignal<number | "">("");
  const [formUnitNumber, setFormUnitNumber] = createSignal("");
  const [formBuilding, setFormBuilding] = createSignal("");
  const [formFloorNumber, setFormFloorNumber] = createSignal<number | "">("");
  const [formTotalFloors, setFormTotalFloors] = createSignal<number | "">("");
  const [formPrice, setFormPrice] = createSignal<number | "">("");
  const [formBuiltUpArea, setFormBuiltUpArea] = createSignal<number | "">("");
  const [formUsableArea, setFormUsableArea] = createSignal<number | "">("");
  const [formStatus, setFormStatus] = createSignal("available");

  // Category adaptive fields
  const [formBedrooms, setFormBedrooms] = createSignal<number | "">("");
  const [formBathrooms, setFormBathrooms] = createSignal<number | "">("");
  const [formBalcony, setFormBalcony] = createSignal<number | "">("");
  const [formIsDuplex, setFormIsDuplex] = createSignal(false);
  const [formParcelNumber, setFormParcelNumber] = createSignal("");
  const [formZoning, setFormZoning] = createSignal("");

  const fetchProperties = async () => {
    setLoading(true);
    try {
      const q = new URLSearchParams();
      if (search().trim()) q.set("q", search().trim());
      if (selectedCategory()) q.set("category", selectedCategory());
      if (selectedProjectId()) q.set("project_id", String(selectedProjectId()));

      const res = await api.get<PropertyGlobalItem[]>(`/api/admin/properties?${q.toString()}`);
      setProperties(res);
    } catch (e) {
      console.error(e);
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
    fetchProperties();
  });

  const selectedTypeObj = () => {
    const tid = formTypeId();
    if (!tid || !meta()) return null;
    return meta()?.property_types.find((t) => t.id === tid);
  };

  const currentCategory = () => selectedTypeObj()?.category || "RESIDENTIAL";

  const handleCreateProperty = async (e: Event) => {
    e.preventDefault();
    if (!formProjectId() || !formTypeId() || formPrice() === "") {
      setFormError("Please select project, property type, and price.");
      return;
    }

    setSubmitting(true);
    setFormError(null);

    const payload: PropertyFormPayload = {
      property_type_id: Number(formTypeId()),
      unit_number: formUnitNumber().trim() || undefined,
      building: formBuilding().trim() || undefined,
      floor_number: formFloorNumber() !== "" ? Number(formFloorNumber()) : undefined,
      total_floors: formTotalFloors() !== "" ? Number(formTotalFloors()) : undefined,
      price: Number(formPrice()),
      built_up_area: formBuiltUpArea() !== "" ? Number(formBuiltUpArea()) : undefined,
      usable_area: formUsableArea() !== "" ? Number(formUsableArea()) : undefined,
      status: formStatus(),
      listing_type: "sale",
      bedroom_count: formBedrooms() !== "" ? Number(formBedrooms()) : undefined,
      bathroom_count: formBathrooms() !== "" ? Number(formBathrooms()) : undefined,
      balcony_count: formBalcony() !== "" ? Number(formBalcony()) : undefined,
      is_duplex: formIsDuplex(),
      parcel_number: formParcelNumber().trim() || undefined,
      zoning: formZoning().trim() || undefined,
      amenities: [],
    };

    try {
      await api.post(`/api/admin/projects/${formProjectId()}/properties`, payload);
      setModalOpen(false);
      resetForm();
      fetchProperties();
    } catch (err: any) {
      setFormError(err.message || "Failed to create property unit");
    } finally {
      setSubmitting(false);
    }
  };

  const resetForm = () => {
    setFormProjectId("");
    setFormTypeId("");
    setFormUnitNumber("");
    setFormBuilding("");
    setFormFloorNumber("");
    setFormTotalFloors("");
    setFormPrice("");
    setFormBuiltUpArea("");
    setFormUsableArea("");
    setFormStatus("available");
    setFormBedrooms("");
    setFormBathrooms("");
    setFormBalcony("");
    setFormIsDuplex(false);
    setFormParcelNumber("");
    setFormZoning("");
    setFormError(null);
  };

  return (
    <div class="space-y-6">
      {/* Header */}
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 pb-4 border-b border-slate-800">
        <div>
          <h1 class="text-2xl font-bold tracking-tight text-white">Global Units & Inventory</h1>
          <p class="text-xs text-slate-400 mt-1">
            Complete inventory catalog across all active developments
          </p>
        </div>

        <Button onClick={() => setModalOpen(true)}>
          <Plus class="w-4 h-4" /> Add Property Unit
        </Button>
      </div>

      {/* Filter and Search Bar */}
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
        <Input
          placeholder="Search by unit number or building..."
          value={search()}
          onInput={(e) => {
            setSearch(e.currentTarget.value);
            debouncedFetchProperties();
          }}
        />

        <Select
          value={selectedCategory()}
          onChange={(e) => {
            setSelectedCategory(e.currentTarget.value);
            fetchProperties();
          }}
          options={[
            { value: "", label: "All Categories" },
            { value: "RESIDENTIAL", label: "Residential" },
            { value: "COMMERCIAL", label: "Commercial" },
            { value: "LAND", label: "Plots & Land" },
          ]}
        />

        <Select
          value={selectedProjectId()}
          onChange={(e) => {
            setSelectedProjectId(e.currentTarget.value ? Number(e.currentTarget.value) : "");
            fetchProperties();
          }}
          options={[
            { value: "", label: "All Projects" },
            // Project options can be fetched from search or meta
          ]}
        />
      </div>

      {/* Properties Table */}
      <div class="bg-slate-900 border border-slate-800 rounded-2xl overflow-hidden shadow-sm">
        <div class="overflow-x-auto">
          <table class="w-full text-left text-xs">
            <thead class="bg-slate-950/70 text-slate-400 uppercase tracking-wider font-semibold border-b border-slate-800">
              <tr>
                <th class="px-6 py-3.5">Unit / Identifier</th>
                <th class="px-6 py-3.5">Project</th>
                <th class="px-6 py-3.5">Category</th>
                <th class="px-6 py-3.5">Type</th>
                <th class="px-6 py-3.5">Price</th>
                <th class="px-6 py-3.5">Status</th>
                <th class="px-6 py-3.5 text-right">Action</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-800/60">
              <For each={properties()}>
                {(prop) => (
                  <tr class="hover:bg-slate-800/40 transition-colors">
                    <td class="px-6 py-4 font-semibold text-slate-100">
                      <A href={`/properties/${prop.id}`} class="hover:text-indigo-400">
                        {prop.unit_title}
                      </A>
                    </td>
                    <td class="px-6 py-4 text-slate-300">
                      <A href={`/projects/${prop.project_id}`} class="hover:underline">
                        {prop.project_name}
                      </A>
                    </td>
                    <td class="px-6 py-4">
                      <Badge
                        variant={
                          prop.category === "RESIDENTIAL"
                            ? "residential"
                            : prop.category === "COMMERCIAL"
                            ? "commercial"
                            : "land"
                        }
                      >
                        {prop.category}
                      </Badge>
                    </td>
                    <td class="px-6 py-4 text-slate-300">{prop.type_name}</td>
                    <td class="px-6 py-4 font-mono font-semibold text-slate-100">
                      ${prop.price.toLocaleString()}
                    </td>
                    <td class="px-6 py-4">
                      <Badge variant={prop.status === "available" ? "success" : "warning"}>
                        {prop.status}
                      </Badge>
                    </td>
                    <td class="px-6 py-4 text-right">
                      <A
                        href={`/properties/${prop.id}`}
                        class="text-indigo-400 hover:text-indigo-300 font-medium inline-flex items-center gap-1"
                      >
                        View & Media <ExternalLink class="w-3 h-3" />
                      </A>
                    </td>
                  </tr>
                )}
              </For>

              {properties().length === 0 && !loading() && (
                <tr>
                  <td colspan="7" class="px-6 py-12 text-center text-slate-500">
                    No property units found. Click "Add Property Unit" to create one.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Category-Adaptive Create Property Modal */}
      <Modal
        isOpen={modalOpen()}
        onClose={() => setModalOpen(false)}
        title="Add Property Unit"
        description="Fill in unit specifications. Fields adapt according to the selected property category."
        maxWidth="2xl"
        footer={
          <>
            <Button variant="ghost" onClick={() => setModalOpen(false)}>
              Cancel
            </Button>
            <Button loading={submitting()} onClick={handleCreateProperty}>
              Save Property Unit
            </Button>
          </>
        }
      >
        <form onSubmit={handleCreateProperty} class="space-y-4">
          {formError() && (
            <div class="p-3 bg-rose-950/60 border border-rose-800 rounded-lg text-xs text-rose-300">
              {formError()}
            </div>
          )}

          {/* Project & Type */}
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <Input
              label="Project ID"
              type="number"
              required
              placeholder="e.g. 1"
              value={formProjectId()}
              onInput={(e) => setFormProjectId(Number(e.currentTarget.value))}
            />

            <Select
              label="Property Type"
              required
              placeholder="Select property type"
              value={formTypeId()}
              onChange={(e) => setFormTypeId(Number(e.currentTarget.value))}
              options={
                meta()?.property_types.map((pt) => ({
                  value: pt.id,
                  label: `${pt.name} (${pt.category})`,
                })) || []
              }
            />
          </div>

          {/* Unit number and building */}
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <Input
              label="Unit / Suite Number"
              placeholder="e.g. 1204"
              value={formUnitNumber()}
              onInput={(e) => setFormUnitNumber(e.currentTarget.value)}
            />
            <Input
              label="Tower / Building Block"
              placeholder="e.g. Tower C"
              value={formBuilding()}
              onInput={(e) => setFormBuilding(e.currentTarget.value)}
            />
          </div>

          {/* Floor and Price */}
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <Input
              label="Floor Number"
              type="number"
              placeholder="e.g. 12"
              value={formFloorNumber()}
              onInput={(e) => setFormFloorNumber(Number(e.currentTarget.value))}
            />
            <Input
              label="Price ($)"
              type="number"
              required
              placeholder="e.g. 450000"
              value={formPrice()}
              onInput={(e) => setFormPrice(Number(e.currentTarget.value))}
            />
            <Select
              label="Status"
              value={formStatus()}
              onChange={(e) => setFormStatus(e.currentTarget.value)}
              options={[
                { value: "available", label: "Available" },
                { value: "reserved", label: "Reserved" },
                { value: "sold", label: "Sold" },
              ]}
            />
          </div>

          {/* Dimensions */}
          <div class="grid grid-cols-2 gap-4">
            <Input
              label="Built-Up Area (sq ft)"
              type="number"
              placeholder="e.g. 1250"
              value={formBuiltUpArea()}
              onInput={(e) => setFormBuiltUpArea(Number(e.currentTarget.value))}
            />
            <Input
              label="Usable Area (Carpet sq ft)"
              type="number"
              placeholder="e.g. 1100"
              value={formUsableArea()}
              onInput={(e) => setFormUsableArea(Number(e.currentTarget.value))}
            />
          </div>

          {/* Category Adaptive Section */}
          <Show when={currentCategory() === "RESIDENTIAL"}>
            <div class="p-4 bg-slate-950/60 border border-slate-800 rounded-xl space-y-4">
              <h4 class="text-xs font-bold uppercase tracking-wider text-indigo-400">
                Residential Specifications
              </h4>
              <div class="grid grid-cols-3 gap-3">
                <Input
                  label="Bedrooms"
                  type="number"
                  placeholder="3"
                  value={formBedrooms()}
                  onInput={(e) => setFormBedrooms(Number(e.currentTarget.value))}
                />
                <Input
                  label="Bathrooms"
                  type="number"
                  placeholder="2"
                  value={formBathrooms()}
                  onInput={(e) => setFormBathrooms(Number(e.currentTarget.value))}
                />
                <Input
                  label="Balconies"
                  type="number"
                  placeholder="1"
                  value={formBalcony()}
                  onInput={(e) => setFormBalcony(Number(e.currentTarget.value))}
                />
              </div>

              <label class="flex items-center gap-2 text-xs text-slate-300 cursor-pointer pt-1">
                <input
                  type="checkbox"
                  checked={formIsDuplex()}
                  onChange={(e) => setFormIsDuplex(e.currentTarget.checked)}
                  class="rounded border-slate-700 bg-slate-900 text-indigo-600 focus:ring-indigo-500/20"
                />
                <span>Is Duplex Penthouse / Multi-Level</span>
              </label>
            </div>
          </Show>

          <Show when={currentCategory() === "LAND"}>
            <div class="p-4 bg-slate-950/60 border border-slate-800 rounded-xl space-y-4">
              <h4 class="text-xs font-bold uppercase tracking-wider text-emerald-400">
                Plot & Land Specifications
              </h4>
              <div class="grid grid-cols-2 gap-4">
                <Input
                  label="Parcel Number"
                  placeholder="e.g. Survey #104/A"
                  value={formParcelNumber()}
                  onInput={(e) => setFormParcelNumber(e.currentTarget.value)}
                />
                <Input
                  label="Zoning Code"
                  placeholder="e.g. R-2 Residential"
                  value={formZoning()}
                  onInput={(e) => setFormZoning(e.currentTarget.value)}
                />
              </div>
            </div>
          </Show>
        </form>
      </Modal>
    </div>
  );
}

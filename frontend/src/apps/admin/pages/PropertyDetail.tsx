import { createSignal, onMount, Show, For } from "solid-js";
import { useParams, useNavigate, A } from "@solidjs/router";
import { ArrowLeft, Trash2, Edit } from "lucide-solid";
import { Button } from "@/shared/components/Button";
import { Badge } from "@/shared/components/Badge";
import { Modal } from "@/shared/components/Modal";
import { Input } from "@/shared/components/Input";
import { Select } from "@/shared/components/Select";
import { Textarea } from "@/shared/components/Textarea";
import { MediaManager } from "@/apps/admin/components/MediaManager";
import { api } from "@/shared/api/client";
import type {
  PropertyDetailResponse,
  PropertyFormPayload,
  MetaOptionsResponse,
} from "@/shared/api/types";

export function PropertyDetail() {
  const params = useParams();
  const navigate = useNavigate();
  const propertyId = () => Number(params.id);

  const [prop, setProp] = createSignal<PropertyDetailResponse | null>(null);
  const [meta, setMeta] = createSignal<MetaOptionsResponse | null>(null);
  const [loading, setLoading] = createSignal(true);

  // Edit Modal State
  const [editModalOpen, setEditModalOpen] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const [editTypeId, setEditTypeId] = createSignal<number | "">("");
  const [editUnitNumber, setEditUnitNumber] = createSignal("");
  const [editBuilding, setEditBuilding] = createSignal("");
  const [editFloor, setEditFloor] = createSignal<number | "">("");
  const [editPrice, setEditPrice] = createSignal<number | "">("");
  const [editListingType, setEditListingType] = createSignal("sale");
  const [editStatus, setEditStatus] = createSignal("active");
  const [editBuiltUpArea, setEditBuiltUpArea] = createSignal<number | "">("");
  const [editBedrooms, setEditBedrooms] = createSignal<number | "">("");
  const [editBathrooms, setEditBathrooms] = createSignal<number | "">("");
  const [editDescription, setEditDescription] = createSignal("");
  const [editAmenities, setEditAmenities] = createSignal<number[]>([]);

  const fetchProperty = async (silent = false) => {
    if (!silent && !prop()) {
      setLoading(true);
    }
    try {
      const res = await api.get<PropertyDetailResponse>(`/api/admin/properties/${propertyId()}`);
      setProp(res);
    } catch (e: any) {
      alert(e.message || "Failed to load property details");
      navigate("/properties");
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
    fetchProperty();
  });

  const availablePropertyTypes = () => {
    if (!meta()?.property_types) return [];
    const pCat = prop()?.category?.toUpperCase();
    const matched = meta()!.property_types.filter(
      (pt) => !pCat || pt.category.toUpperCase() === pCat
    );
    return (matched.length > 0 ? matched : meta()!.property_types).map((pt) => ({
      value: pt.id,
      label: `${pt.name} (${pt.category})`,
    }));
  };

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

  const openEditModal = () => {
    const p = prop();
    if (!p) return;
    setEditTypeId(p.property_type_id);
    setEditUnitNumber(p.unit_number || "");
    setEditBuilding(p.building || "");
    setEditFloor(p.floor_number ?? "");
    setEditPrice(p.price);
    setEditListingType(p.listing_type || "sale");
    setEditStatus(p.status ? (p.status === "available" ? "active" : p.status) : "active");
    setEditBuiltUpArea(p.built_up_area ?? "");
    setEditBedrooms(p.bedroom_count ?? "");
    setEditBathrooms(p.bathroom_count ?? "");
    setEditDescription(p.description || "");
    setEditAmenities(p.amenities ? [...p.amenities] : []);
    setError(null);
    setEditModalOpen(true);
  };

  const toggleAmenity = (id: number) => {
    setEditAmenities((prev) =>
      prev.includes(id) ? prev.filter((a) => a !== id) : [...prev, id]
    );
  };

  const handleUpdate = async (e: Event) => {
    e.preventDefault();
    if (!editTypeId() || editPrice() === "") {
      setError("Please select property type and specify listing price.");
      return;
    }
    setSubmitting(true);
    setError(null);

    const payload: PropertyFormPayload = {
      property_type_id: Number(editTypeId()),
      category: prop()?.category || null,
      unit_number: editUnitNumber().trim() || null,
      building: editBuilding().trim() || null,
      floor_number: editFloor() !== "" ? Number(editFloor()) : null,
      total_floors: prop()?.total_floors ?? null,
      built_up_area: editBuiltUpArea() !== "" ? Number(editBuiltUpArea()) : null,
      usable_area: prop()?.usable_area ?? null,
      description: editDescription().trim() || null,
      listing_type: editListingType(),
      status: editStatus(),
      price: Number(editPrice()),
      billing_period: prop()?.billing_period ?? null,
      bedroom_count: editBedrooms() !== "" ? Number(editBedrooms()) : null,
      bathroom_count: editBathrooms() !== "" ? Number(editBathrooms()) : null,
      balcony_count: prop()?.balcony_count ?? null,
      is_duplex: prop()?.is_duplex ?? null,
      parking: prop()?.parking ?? null,
      parcel_number: prop()?.parcel_number ?? null,
      zoning: prop()?.zoning ?? null,
      approval_status: prop()?.approval_status ?? null,
      development_status: prop()?.development_status ?? null,
      amenities: editAmenities(),
    };

    try {
      await api.put(`/api/admin/properties/${propertyId()}`, payload);
      setEditModalOpen(false);
      fetchProperty(true);
    } catch (err: any) {
      setError(err.message || "Failed to update property unit");
    } finally {
      setSubmitting(false);
    }
  };

  const handleDelete = async () => {
    if (!confirm("Delete this property unit? This action cannot be undone.")) return;
    try {
      await api.delete(`/api/admin/properties/${propertyId()}`);
      navigate("/properties");
    } catch (e: any) {
      alert(e.message || "Failed to delete property");
    }
  };

  return (
    <div class="space-y-6">
      <div class="flex items-center justify-between">
        <A
          href="/properties"
          class="inline-flex items-center gap-1.5 text-xs font-medium text-slate-400 hover:text-white transition-colors"
        >
          <ArrowLeft class="w-3.5 h-3.5" /> Back to Units Inventory
        </A>

        <div class="flex items-center gap-2">
          <Button variant="secondary" size="sm" onClick={openEditModal}>
            <Edit class="w-3.5 h-3.5" /> Edit Unit
          </Button>
          <Button variant="danger" size="sm" onClick={handleDelete}>
            <Trash2 class="w-3.5 h-3.5" /> Delete Unit
          </Button>
        </div>
      </div>

      <Show when={!loading() && prop()}>
        {(p) => {
          const title = p().unit_number
            ? `Unit ${p().unit_number}${p().building ? `, ${p().building}` : ""}`
            : p().building || `Unit #${p().id}`;

          return (
            <div class="space-y-8">
              {/* Header Box */}
              <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 sm:p-8">
                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                  <div>
                    <div class="flex items-center gap-3">
                      <h1 class="text-2xl font-bold text-white">{title}</h1>
                      <Badge variant={p().status === "available" || p().status === "active" ? "success" : "warning"}>
                        {formatListingStatus(p().status, p().listing_type)}
                      </Badge>
                      <Badge variant="default">{p().category}</Badge>
                    </div>
                    <p class="text-xs text-slate-400 mt-1">
                      Part of Project #
                      <A href={`/projects/${p().project_id}`} class="text-indigo-400 hover:underline">
                        {p().project_id}
                      </A>
                    </p>
                  </div>

                  <div class="text-right">
                    <div class="text-3xl font-bold font-mono text-emerald-400">
                      ${p().price.toLocaleString()}
                    </div>
                    <div class="text-[11px] text-slate-500 uppercase tracking-wider font-semibold">
                      Listing Price ({p().listing_type})
                    </div>
                  </div>
                </div>

                {/* Specs Bar */}
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-4 mt-8 pt-6 border-t border-slate-800">
                  <div>
                    <span class="text-[11px] text-slate-500 uppercase">Property Type</span>
                    <p class="text-sm font-semibold text-slate-200">{p().type_name}</p>
                  </div>
                  <div>
                    <span class="text-[11px] text-slate-500 uppercase">Built-Up Area</span>
                    <p class="text-sm font-semibold text-slate-200">
                      {p().built_up_area ? `${p().built_up_area} sq ft` : "Not specified"}
                    </p>
                  </div>
                  <div>
                    <span class="text-[11px] text-slate-500 uppercase">Floor / Level</span>
                    <p class="text-sm font-semibold text-slate-200">
                      {p().floor_number !== undefined && p().floor_number !== null
                        ? `Floor ${p().floor_number}`
                        : "N/A"}
                    </p>
                  </div>
                  <div>
                    <span class="text-[11px] text-slate-500 uppercase">Bedrooms / Bathrooms</span>
                    <p class="text-sm font-semibold text-slate-200">
                      {p().bedroom_count ? `${p().bedroom_count} BHK` : "N/A"}
                      {p().bathroom_count ? ` • ${p().bathroom_count} Bath` : ""}
                    </p>
                  </div>
                </div>

                {p().description && (
                  <div class="mt-6 pt-6 border-t border-slate-800">
                    <span class="text-[11px] text-slate-500 uppercase">Description</span>
                    <p class="text-sm text-slate-300 mt-1 whitespace-pre-line leading-relaxed">
                      {p().description}
                    </p>
                  </div>
                )}
              </div>

              {/* Media Gallery */}
              <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6">
                <h2 class="text-base font-bold text-white mb-4">Unit Photos & Media</h2>
                <MediaManager
                  uploadEndpoint={`/api/admin/properties/${propertyId()}/media`}
                  reorderEndpoint={`/api/admin/properties/${propertyId()}/media/reorder`}
                  deleteEndpoint={(id) => `/api/admin/properties/${propertyId()}/media/${id}`}
                  media={p().media}
                  onRefresh={() => fetchProperty(true)}
                />
              </div>
            </div>
          );
        }}
      </Show>

      {/* Edit Property Modal */}
      <Modal
        isOpen={editModalOpen()}
        onClose={() => setEditModalOpen(false)}
        title="Edit Property Unit"
        description="Update pricing, specifications, listing status, or unit details."
        footer={
          <>
            <Button variant="ghost" onClick={() => setEditModalOpen(false)}>
              Cancel
            </Button>
            <Button loading={submitting()} onClick={handleUpdate}>
              Save Changes
            </Button>
          </>
        }
      >
        <form onSubmit={handleUpdate} class="space-y-4">
          {error() && (
            <div class="p-3 bg-rose-950/60 border border-rose-800 rounded-lg text-xs text-rose-300">
              {error()}
            </div>
          )}

          <Select
            label="Property Type"
            required
            placeholder="Select unit type"
            value={editTypeId()}
            onChange={(e) => setEditTypeId(Number(e.currentTarget.value))}
            options={availablePropertyTypes()}
          />

          <div class="grid grid-cols-2 gap-4">
            <Input
              label="Unit / Flat Number"
              placeholder="e.g. 402B"
              value={editUnitNumber()}
              onInput={(e) => setEditUnitNumber(e.currentTarget.value)}
            />
            <Input
              label="Tower / Building"
              placeholder="e.g. Tower B"
              value={editBuilding()}
              onInput={(e) => setEditBuilding(e.currentTarget.value)}
            />
          </div>

          <div class="grid grid-cols-2 gap-4">
            <Input
              label="Floor Number"
              type="number"
              placeholder="e.g. 4"
              value={editFloor()}
              onInput={(e) => setEditFloor(Number(e.currentTarget.value))}
            />
            <Input
              label="Listing Price ($)"
              type="number"
              required
              placeholder="e.g. 350000"
              value={editPrice()}
              onInput={(e) => setEditPrice(Number(e.currentTarget.value))}
            />
          </div>

          <div class="grid grid-cols-3 gap-4">
            <Select
              label="Listing Type"
              value={editListingType()}
              onChange={(e) => {
                const newType = e.currentTarget.value;
                setEditListingType(newType);
                const validOptions = getStatusOptions(newType).map((o) => o.value);
                if (!validOptions.includes(editStatus())) {
                  setEditStatus("active");
                }
              }}
              options={[
                { value: "sale", label: "For Sale" },
                { value: "rent", label: "For Rent" },
                { value: "lease", label: "For Lease" },
              ]}
            />
            <Select
              label="Status"
              value={editStatus()}
              onChange={(e) => setEditStatus(e.currentTarget.value)}
              options={getStatusOptions(editListingType())}
            />
            <Input
              label="Built-Up Area (sq ft)"
              type="number"
              placeholder="e.g. 1450"
              value={editBuiltUpArea()}
              onInput={(e) => setEditBuiltUpArea(Number(e.currentTarget.value))}
            />
          </div>

          <div class="grid grid-cols-2 gap-4">
            <Input
              label="Bedrooms"
              type="number"
              placeholder="e.g. 3"
              value={editBedrooms()}
              onInput={(e) => setEditBedrooms(Number(e.currentTarget.value))}
            />
            <Input
              label="Bathrooms"
              type="number"
              placeholder="e.g. 2"
              value={editBathrooms()}
              onInput={(e) => setEditBathrooms(Number(e.currentTarget.value))}
            />
          </div>

          <Textarea
            label="Unit Description"
            placeholder="Special features, views, balconies, or furnishing..."
            rows={3}
            value={editDescription()}
            onInput={(e) => setEditDescription(e.currentTarget.value)}
          />

          <div>
            <label class="block text-xs font-medium text-slate-300 mb-1.5">
              Unit Amenities
            </label>
            <div class="grid grid-cols-2 sm:grid-cols-3 gap-2 max-h-40 overflow-y-auto p-2 bg-slate-950 border border-slate-800 rounded-lg">
              <For each={meta()?.amenities || []}>
                {(amenity) => {
                  const isChecked = () => editAmenities().includes(amenity.id);
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
    </div>
  );
}

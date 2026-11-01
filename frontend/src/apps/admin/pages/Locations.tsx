import { createSignal, onMount, For } from "solid-js";
import { MapPin, Plus, Trash2 } from "lucide-solid";
import { Button } from "@/shared/components/Button";
import { Input } from "@/shared/components/Input";
import { Modal } from "@/shared/components/Modal";
import { api } from "@/shared/api/client";
import { LocationItem, CreateOrUpdateLocationPayload } from "@/shared/api/types";
import { debounce } from "@/shared/utils/debounce";

export function Locations() {
  const [locations, setLocations] = createSignal<LocationItem[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [search, setSearch] = createSignal("");

  const debouncedFetchLocations = debounce(() => {
    fetchLocations();
  }, 300);

  // Modal
  const [modalOpen, setModalOpen] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const [formAddress, setFormAddress] = createSignal("");
  const [formArea, setFormArea] = createSignal("");
  const [formCity, setFormCity] = createSignal("");
  const [formStateOrProvince, setFormStateOrProvince] = createSignal("");

  const fetchLocations = async () => {
    setLoading(true);
    try {
      const q = search().trim() ? `?q=${encodeURIComponent(search().trim())}` : "";
      const res = await api.get<LocationItem[]>(`/api/admin/locations${q}`);
      setLocations(res);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  onMount(fetchLocations);

  const handleCreate = async (e: Event) => {
    e.preventDefault();
    if (!formAddress().trim()) {
      setError("Formatted address is required.");
      return;
    }

    setSubmitting(true);
    setError(null);

    try {
      const payload: CreateOrUpdateLocationPayload = {
        formatted_address: formAddress().trim(),
        area: formArea().trim() || undefined,
        city: formCity().trim() || undefined,
        state_or_province: formStateOrProvince().trim() || undefined,
      };

      await api.post("/api/admin/locations", payload);

      setModalOpen(false);
      resetForm();
      fetchLocations();
    } catch (err: any) {
      setError(err.message || "Failed to add location");
    } finally {
      setSubmitting(false);
    }
  };

  const handleDelete = async (id: number) => {
    if (!confirm("Are you sure you want to delete this location?")) return;
    try {
      await api.delete(`/api/admin/locations/${id}`);
      fetchLocations();
    } catch (e: any) {
      alert(e.message || "Failed to delete location");
    }
  };

  const resetForm = () => {
    setFormAddress("");
    setFormArea("");
    setFormCity("");
    setFormStateOrProvince("");
    setError(null);
  };

  return (
    <div class="space-y-6">
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 class="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <MapPin class="w-5 h-5 text-indigo-400" />
            Geographic Locations
          </h1>
          <p class="text-xs text-slate-400 mt-1">
            Manage standardized addresses and regional mappings across all developments.
          </p>
        </div>

        <Button
          onClick={() => {
            resetForm();
            setModalOpen(true);
          }}
          class="flex items-center gap-2"
        >
          <Plus class="w-4 h-4" />
          Add Location
        </Button>
      </div>

      <div class="bg-slate-900 border border-slate-800 rounded-2xl p-4 shadow-sm flex items-center justify-between gap-4">
        <div class="relative w-full max-w-sm">
          <Input
            placeholder="Filter locations..."
            value={search()}
            onInput={(e) => {
              setSearch(e.currentTarget.value);
              debouncedFetchLocations();
            }}
          />
        </div>
      </div>

      <div class="bg-slate-900 border border-slate-800 rounded-2xl overflow-hidden shadow-sm">
        <div class="overflow-x-auto">
          <table class="w-full text-left text-xs">
            <thead class="bg-slate-950 text-slate-400 uppercase tracking-wider font-semibold border-b border-slate-800">
              <tr>
                <th class="px-6 py-3.5">Formatted Address</th>
                <th class="px-6 py-3.5">Area / Neighborhood</th>
                <th class="px-6 py-3.5">City</th>
                <th class="px-6 py-3.5">State / Province</th>
                <th class="px-6 py-3.5 text-right">Action</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-800/60">
              <For each={locations()}>
                {(loc) => (
                  <tr class="hover:bg-slate-800/40 transition-colors">
                    <td class="px-6 py-4 font-semibold text-slate-100 flex items-center gap-2">
                      <MapPin class="w-3.5 h-3.5 text-slate-500 shrink-0" />
                      {loc.formatted_address}
                    </td>
                    <td class="px-6 py-4 text-slate-300">{loc.area || "-"}</td>
                    <td class="px-6 py-4 text-slate-300">{loc.city || "-"}</td>
                    <td class="px-6 py-4 text-slate-300">{loc.state_or_province || "-"}</td>
                    <td class="px-6 py-4 text-right">
                      <Button
                        variant="ghost"
                        size="sm"
                        class="text-rose-400 hover:text-rose-300"
                        onClick={() => handleDelete(loc.id)}
                      >
                        <Trash2 class="w-3.5 h-3.5" />
                      </Button>
                    </td>
                  </tr>
                )}
              </For>
              {locations().length === 0 && !loading() && (
                <tr>
                  <td colspan="5" class="px-6 py-12 text-center text-slate-500">
                    No locations found. Click "Add Location" to register one.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Modal */}
      <Modal
        isOpen={modalOpen()}
        onClose={() => setModalOpen(false)}
        title="Add Geographic Location"
        footer={
          <>
            <Button variant="ghost" onClick={() => setModalOpen(false)}>
              Cancel
            </Button>
            <Button loading={submitting()} onClick={handleCreate}>
              Save Location
            </Button>
          </>
        }
      >
        <form onSubmit={handleCreate} class="space-y-4">
          {error() && (
            <div class="p-3 bg-rose-950/60 border border-rose-800 rounded-lg text-xs text-rose-300">
              {error()}
            </div>
          )}

          <Input
            label="Full Formatted Address"
            required
            placeholder="e.g. 100 Financial Way, Downtown"
            value={formAddress()}
            onInput={(e) => setFormAddress(e.currentTarget.value)}
          />

          <div class="grid grid-cols-3 gap-3">
            <Input
              label="Area / Neighborhood"
              placeholder="Downtown"
              value={formArea()}
              onInput={(e) => setFormArea(e.currentTarget.value)}
            />
            <Input
              label="City"
              placeholder="Metropolis"
              value={formCity()}
              onInput={(e) => setFormCity(e.currentTarget.value)}
            />
            <Input
              label="State / Province"
              placeholder="CA"
              value={formStateOrProvince()}
              onInput={(e) => setFormStateOrProvince(e.currentTarget.value)}
            />
          </div>
        </form>
      </Modal>
    </div>
  );
}

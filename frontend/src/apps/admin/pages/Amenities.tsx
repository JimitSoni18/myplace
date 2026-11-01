import { createSignal, onMount, For } from "solid-js";
import { Sparkles, Plus, Trash2 } from "lucide-solid";
import { Button } from "@/shared/components/Button";
import { Input } from "@/shared/components/Input";
import { Badge } from "@/shared/components/Badge";
import { Modal } from "@/shared/components/Modal";
import { api } from "@/shared/api/client";
import { AmenityItem } from "@/shared/api/types";

export function Amenities() {
  const [amenities, setAmenities] = createSignal<AmenityItem[]>([]);

  // Modal
  const [modalOpen, setModalOpen] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [formName, setFormName] = createSignal("");

  const fetchAmenities = async () => {
    try {
      const res = await api.get<AmenityItem[]>("/api/admin/amenities");
      setAmenities(res);
    } catch (e) {
      console.error(e);
    }
  };

  onMount(fetchAmenities);

  const handleToggle = async (id: number) => {
    try {
      await api.post(`/api/admin/amenities/${id}/toggle`);
      fetchAmenities();
    } catch (e: any) {
      alert(e.message || "Failed to toggle amenity");
    }
  };

  const handleCreate = async (e: Event) => {
    e.preventDefault();
    if (!formName().trim()) return;

    setSubmitting(true);
    setError(null);

    try {
      await api.post("/api/admin/amenities", {
        name: formName().trim(),
        is_active: true,
      });

      setModalOpen(false);
      setFormName("");
      fetchAmenities();
    } catch (err: any) {
      setError(err.message || "Failed to create amenity");
    } finally {
      setSubmitting(false);
    }
  };

  const handleDelete = async (id: number) => {
    if (!confirm("Are you sure you want to delete this amenity?")) return;
    try {
      await api.delete(`/api/admin/amenities/${id}`);
      fetchAmenities();
    } catch (e: any) {
      alert(e.message || "Failed to delete amenity");
    }
  };

  return (
    <div class="space-y-6">
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 pb-4 border-b border-slate-800">
        <div>
          <h1 class="text-2xl font-bold tracking-tight text-white">Project Amenities</h1>
          <p class="text-xs text-slate-400 mt-1">
            Master catalog of lifestyle, security, and structural amenities
          </p>
        </div>

        <Button onClick={() => setModalOpen(true)}>
          <Plus class="w-4 h-4" /> Add Amenity
        </Button>
      </div>

      <div class="bg-slate-900 border border-slate-800 rounded-2xl overflow-hidden shadow-sm">
        <div class="overflow-x-auto">
          <table class="w-full text-left text-xs">
            <thead class="bg-slate-950 text-slate-400 uppercase tracking-wider font-semibold border-b border-slate-800">
              <tr>
                <th class="px-6 py-3.5">Amenity Name</th>
                <th class="px-6 py-3.5">Linked Projects</th>
                <th class="px-6 py-3.5">Status</th>
                <th class="px-6 py-3.5 text-right">Actions</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-800/60">
              <For each={amenities()}>
                {(amenity) => (
                  <tr class="hover:bg-slate-800/40 transition-colors">
                    <td class="px-6 py-4 font-semibold text-slate-100 flex items-center gap-2">
                      <Sparkles class="w-3.5 h-3.5 text-indigo-400 shrink-0" />
                      {amenity.name}
                    </td>
                    <td class="px-6 py-4 text-slate-300">
                      <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-slate-800 text-xs text-slate-300 font-medium">
                        {amenity.projects_count} {amenity.projects_count === 1 ? "project" : "projects"}
                      </span>
                    </td>
                    <td class="px-6 py-4">
                      <Badge variant={amenity.is_active ? "success" : "default"}>
                        {amenity.is_active ? "Active" : "Disabled"}
                      </Badge>
                    </td>
                    <td class="px-6 py-4 text-right space-x-2">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleToggle(amenity.id)}
                        class={amenity.is_active ? "text-amber-400" : "text-emerald-400"}
                      >
                        {amenity.is_active ? "Disable" : "Enable"}
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        class="text-rose-400 hover:text-rose-300"
                        onClick={() => handleDelete(amenity.id)}
                      >
                        <Trash2 class="w-3.5 h-3.5" />
                      </Button>
                    </td>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
        </div>
      </div>

      <Modal
        isOpen={modalOpen()}
        onClose={() => setModalOpen(false)}
        title="Add New Amenity"
        footer={
          <>
            <Button variant="ghost" onClick={() => setModalOpen(false)}>
              Cancel
            </Button>
            <Button loading={submitting()} onClick={handleCreate}>
              Save Amenity
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
            label="Amenity Name"
            required
            placeholder="e.g. Olympic Size Swimming Pool"
            value={formName()}
            onInput={(e) => setFormName(e.currentTarget.value)}
          />
        </form>
      </Modal>
    </div>
  );
}

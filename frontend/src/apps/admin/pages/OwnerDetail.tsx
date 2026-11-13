import { createSignal, onMount, Show, For } from "solid-js";
import { useParams, useNavigate, A } from "@solidjs/router";
import { Mail, Phone, Globe, ArrowLeft, Camera, ExternalLink } from "lucide-solid";
import { Button } from "@/shared/components/Button";
import { Badge } from "@/shared/components/Badge";
import { Modal } from "@/shared/components/Modal";
import { Input } from "@/shared/components/Input";
import { Textarea } from "@/shared/components/Textarea";
import { api } from "@/shared/api/client";
import { OwnerDetailResponse } from "@/shared/api/types";

export function OwnerDetail() {
  const params = useParams();
  const navigate = useNavigate();
  const ownerId = () => Number(params.id);

  const [owner, setOwner] = createSignal<OwnerDetailResponse | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [imgUploading, setImgUploading] = createSignal(false);

  // Edit modal
  const [editModalOpen, setEditModalOpen] = createSignal(false);
  const [editName, setEditName] = createSignal("");
  const [editEmail, setEditEmail] = createSignal("");
  const [editPhone, setEditPhone] = createSignal("");
  const [editWebsite, setEditWebsite] = createSignal("");
  const [editBio, setEditBio] = createSignal("");
  const [editActive, setEditActive] = createSignal(true);
  const [editSubmitting, setEditSubmitting] = createSignal(false);
  const [editError, setEditError] = createSignal<string | null>(null);

  const fetchOwner = async () => {
    setLoading(true);
    try {
      const res = await api.get<OwnerDetailResponse>(`/api/admin/project_owners/${ownerId()}`);
      setOwner(res);
      setEditName(res.name);
      setEditEmail(res.email || "");
      setEditPhone(res.phone || "");
      setEditWebsite(res.website || "");
      setEditBio(res.bio || "");
      setEditActive(res.active);
    } catch (e: any) {
      alert(e.message || "Failed to load owner");
      navigate("/owners");
    } finally {
      setLoading(false);
    }
  };

  onMount(fetchOwner);

  const handleUploadImage = async (files: FileList | null) => {
    if (!files || files.length === 0) return;
    setImgUploading(true);
    const formData = new FormData();
    formData.append("image", files[0]);

    try {
      await api.upload(`/api/admin/project_owners/${ownerId()}/image`, formData);
      fetchOwner();
    } catch (e: any) {
      alert(e.message || "Failed to upload image");
    } finally {
      setImgUploading(false);
    }
  };

  const handleUpdate = async (e: Event) => {
    e.preventDefault();
    if (!editName().trim()) return;

    setEditSubmitting(true);
    setEditError(null);

    try {
      await api.put(`/api/admin/project_owners/${ownerId()}`, {
        name: editName().trim(),
        email: editEmail().trim() || undefined,
        phone: editPhone().trim() || undefined,
        website: editWebsite().trim() || undefined,
        bio: editBio().trim() || undefined,
        active: editActive(),
      });
      setEditModalOpen(false);
      fetchOwner();
    } catch (err: any) {
      setEditError(err.message || "Failed to update owner");
    } finally {
      setEditSubmitting(false);
    }
  };

  return (
    <div class="space-y-6">
      <div class="flex items-center justify-between">
        <A
          href="/owners"
          class="inline-flex items-center gap-1.5 text-xs font-medium text-slate-400 hover:text-white transition-colors"
        >
          <ArrowLeft class="w-3.5 h-3.5" /> Back to Owners Directory
        </A>

        <Button variant="secondary" size="sm" onClick={() => setEditModalOpen(true)}>
          Edit Profile
        </Button>
      </div>

      <Show when={!loading() && owner()}>
        {(o) => (
          <div class="space-y-8">
            {/* Header Box */}
            <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 sm:p-8">
              <div class="flex flex-col sm:flex-row sm:items-center gap-6">
                {/* Profile Image & Uploader */}
                <div class="relative group w-24 h-24 rounded-2xl bg-slate-800 border-2 border-slate-700 overflow-hidden flex items-center justify-center shrink-0">
                  {(o().profile_image_url || o().profile_thumb_url) ? (
                    <img src={(o().profile_image_url || o().profile_thumb_url)!} alt={o().name} class="w-full h-full object-cover" />
                  ) : (
                    <span class="text-3xl font-bold text-slate-400">{o().name.charAt(0)}</span>
                  )}

                  <label
                    for="avatar-input"
                    class="absolute inset-0 bg-slate-950/70 opacity-0 group-hover:opacity-100 transition-opacity flex flex-col items-center justify-center text-xs text-white cursor-pointer"
                  >
                    <Camera class="w-5 h-5 mb-1" />
                    <span>{imgUploading() ? "..." : "Change"}</span>
                  </label>
                  <input
                    type="file"
                    id="avatar-input"
                    accept="image/*"
                    class="hidden"
                    onChange={(e) => handleUploadImage(e.currentTarget.files)}
                  />
                </div>

                <div class="flex-1">
                  <div class="flex items-center gap-3">
                    <h1 class="text-2xl font-bold text-white">{o().name}</h1>
                    <Badge variant={o().active ? "success" : "danger"}>
                      {o().active ? "Active" : "Inactive"}
                    </Badge>
                  </div>
                  <p class="text-xs font-mono text-slate-400 mt-1">Slug: /{o().slug}</p>

                  <div class="flex flex-wrap items-center gap-4 text-xs text-slate-400 mt-4">
                    {o().email && (
                      <span class="inline-flex items-center gap-1.5">
                        <Mail class="w-3.5 h-3.5 text-slate-500" /> {o().email}
                      </span>
                    )}
                    {o().phone && (
                      <span class="inline-flex items-center gap-1.5">
                        <Phone class="w-3.5 h-3.5 text-slate-500" /> {o().phone}
                      </span>
                    )}
                    {o().website && (
                      <a
                        href={o().website!}
                        target="_blank"
                        rel="noreferrer"
                        class="inline-flex items-center gap-1.5 text-indigo-400 hover:underline"
                      >
                        <Globe class="w-3.5 h-3.5 text-slate-500" /> {o().website}
                      </a>
                    )}
                  </div>
                </div>
              </div>

              {o().bio && (
                <div class="mt-6 pt-6 border-t border-slate-800 text-xs text-slate-300 leading-relaxed">
                  <h4 class="text-xs font-bold uppercase tracking-wider text-slate-500 mb-2">
                    Company Bio
                  </h4>
                  {o().bio}
                </div>
              )}
            </div>

            {/* Projects by Owner */}
            <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 space-y-4">
              <h2 class="text-base font-bold text-white">
                Projects by {o().name} ({o().projects.length})
              </h2>

              <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                <For each={o().projects}>
                  {(proj) => (
                    <div class="p-4 bg-slate-950 border border-slate-800 rounded-xl hover:border-slate-700 transition-colors flex flex-col justify-between">
                      <div>
                        <div class="flex items-center justify-between gap-2 mb-2">
                          <h3 class="text-sm font-bold text-white truncate">{proj.name}</h3>
                          <Badge variant="default">{proj.project_type_name}</Badge>
                        </div>
                        <p class="text-xs text-slate-400 truncate">{proj.area ? `${proj.area}, ${proj.city}` : proj.city || proj.formatted_address}</p>
                      </div>

                      <div class="flex items-center justify-between pt-3 mt-4 border-t border-slate-850">
                        <span class="text-xs text-slate-500 font-mono">
                          {proj.configurations_count} typologies
                        </span>
                        <A href={`/projects/${proj.id}`}>
                          <Button variant="secondary" size="sm">
                            View <ExternalLink class="w-3 h-3 ml-1" />
                          </Button>
                        </A>
                      </div>
                    </div>
                  )}
                </For>
                {o().projects.length === 0 && (
                  <div class="col-span-3 text-center py-8 text-xs text-slate-500">
                    No projects registered for this owner yet.
                  </div>
                )}
              </div>
            </div>
          </div>
        )}
      </Show>

      {/* Edit Modal */}
      <Modal
        isOpen={editModalOpen()}
        onClose={() => setEditModalOpen(false)}
        title="Edit Partner Profile"
        footer={
          <>
            <Button variant="ghost" onClick={() => setEditModalOpen(false)}>
              Cancel
            </Button>
            <Button loading={editSubmitting()} onClick={handleUpdate}>
              Save Changes
            </Button>
          </>
        }
      >
        <form onSubmit={handleUpdate} class="space-y-4">
          {editError() && (
            <div class="p-3 bg-rose-950/60 border border-rose-800 rounded-lg text-xs text-rose-300">
              {editError()}
            </div>
          )}

          <Input
            label="Company Name"
            required
            value={editName()}
            onInput={(e) => setEditName(e.currentTarget.value)}
          />

          <div class="grid grid-cols-2 gap-4">
            <Input
              label="Contact Email"
              type="email"
              value={editEmail()}
              onInput={(e) => setEditEmail(e.currentTarget.value)}
            />
            <Input
              label="Phone Number"
              value={editPhone()}
              onInput={(e) => setEditPhone(e.currentTarget.value)}
            />
          </div>

          <Input
            label="Website"
            value={editWebsite()}
            onInput={(e) => setEditWebsite(e.currentTarget.value)}
          />

          <Textarea
            label="Bio"
            value={editBio()}
            onInput={(e) => setEditBio(e.currentTarget.value)}
          />
        </form>
      </Modal>
    </div>
  );
}

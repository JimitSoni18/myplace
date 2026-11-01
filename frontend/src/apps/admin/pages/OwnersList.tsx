import { createSignal, onMount, For, Show } from "solid-js";
import { A, useSearchParams } from "@solidjs/router";
import { Users, Plus, Mail, Phone, Globe, ExternalLink } from "lucide-solid";
import { Button } from "@/shared/components/Button";
import { Input } from "@/shared/components/Input";
import { Badge } from "@/shared/components/Badge";
import { Modal } from "@/shared/components/Modal";
import { Textarea } from "@/shared/components/Textarea";
import { api } from "@/shared/api/client";
import { OwnerListItem } from "@/shared/api/types";
import { debounce } from "@/shared/utils/debounce";

export function OwnersList() {
  const [params] = useSearchParams();
  const [owners, setOwners] = createSignal<OwnerListItem[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [search, setSearch] = createSignal("");

  const debouncedFetchOwners = debounce(() => {
    fetchOwners();
  }, 300);

  // Modal
  const [modalOpen, setModalOpen] = createSignal(params.action === "new");
  const [submitting, setSubmitting] = createSignal(false);
  const [formError, setFormError] = createSignal<string | null>(null);

  const [formName, setFormName] = createSignal("");
  const [formUsername, setFormUsername] = createSignal("");
  const [formPassword, setFormPassword] = createSignal("");
  const [formEmail, setFormEmail] = createSignal("");
  const [formPhone, setFormPhone] = createSignal("");
  const [formWebsite, setFormWebsite] = createSignal("");
  const [formBio, setFormBio] = createSignal("");

  const fetchOwners = async () => {
    setLoading(true);
    try {
      const q = search().trim() ? `?q=${encodeURIComponent(search().trim())}` : "";
      const res = await api.get<OwnerListItem[]>(`/api/admin/project_owners${q}`);
      setOwners(res);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  onMount(fetchOwners);

  const handleToggleActive = async (id: number) => {
    try {
      await api.post(`/api/admin/project_owners/${id}/toggle-active`);
      fetchOwners();
    } catch (e: any) {
      alert(e.message || "Failed to toggle status");
    }
  };

  const handleCreateOwner = async (e: Event) => {
    e.preventDefault();
    if (!formName().trim() || !formUsername().trim() || !formPassword()) {
      setFormError("Name, username, and password are required.");
      return;
    }

    setSubmitting(true);
    setFormError(null);

    try {
      await api.post("/api/admin/project_owners", {
        name: formName().trim(),
        username: formUsername().trim(),
        password: formPassword(),
        email: formEmail().trim() || undefined,
        phone: formPhone().trim() || undefined,
        website: formWebsite().trim() || undefined,
        bio: formBio().trim() || undefined,
        active: true,
      });

      setModalOpen(false);
      resetForm();
      fetchOwners();
    } catch (err: any) {
      setFormError(err.message || "Failed to register partner owner");
    } finally {
      setSubmitting(false);
    }
  };

  const resetForm = () => {
    setFormName("");
    setFormUsername("");
    setFormPassword("");
    setFormEmail("");
    setFormPhone("");
    setFormWebsite("");
    setFormBio("");
    setFormError(null);
  };

  return (
    <div class="space-y-6">
      {/* Header */}
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 pb-4 border-b border-slate-800">
        <div>
          <h1 class="text-2xl font-bold tracking-tight text-white">Project Owners Directory</h1>
          <p class="text-xs text-slate-400 mt-1">
            Manage authorized builder partners and real estate development companies
          </p>
        </div>

        <Button onClick={() => setModalOpen(true)}>
          <Plus class="w-4 h-4" /> Register Partner Owner
        </Button>
      </div>

      {/* Search */}
      <div class="max-w-md">
        <Input
          placeholder="Search owners by company or brand name..."
          value={search()}
          onInput={(e) => {
            setSearch(e.currentTarget.value);
            debouncedFetchOwners();
          }}
        />
      </div>

      {/* Owners Cards */}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
        <For each={owners()}>
          {(owner) => (
            <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 flex flex-col justify-between hover:border-slate-700 transition-all shadow-sm">
              <div>
                <div class="flex items-start justify-between gap-3">
                  <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-xl bg-slate-800 border border-slate-700 flex items-center justify-center font-bold text-slate-200 overflow-hidden shrink-0">
                      {owner.profile_thumb_url ? (
                        <img src={owner.profile_thumb_url} alt={owner.name} class="w-full h-full object-cover" />
                      ) : (
                        owner.name.charAt(0)
                      )}
                    </div>
                    <div>
                      <h3 class="text-sm font-bold text-white hover:text-indigo-400 transition-colors">
                        <A href={`/owners/${owner.id}`}>{owner.name}</A>
                      </h3>
                      <span class="text-[11px] text-slate-500 font-mono">/{owner.slug}</span>
                    </div>
                  </div>

                  <Badge variant={owner.active ? "success" : "danger"}>
                    {owner.active ? "Active" : "Inactive"}
                  </Badge>
                </div>

                <div class="mt-4 space-y-1.5 text-xs text-slate-400">
                  {owner.email && (
                    <div class="flex items-center gap-2">
                      <Mail class="w-3.5 h-3.5 text-slate-500 shrink-0" />
                      <span class="truncate">{owner.email}</span>
                    </div>
                  )}
                  {owner.phone && (
                    <div class="flex items-center gap-2">
                      <Phone class="w-3.5 h-3.5 text-slate-500 shrink-0" />
                      <span>{owner.phone}</span>
                    </div>
                  )}
                  {owner.website && (
                    <div class="flex items-center gap-2">
                      <Globe class="w-3.5 h-3.5 text-slate-500 shrink-0" />
                      <a href={owner.website} target="_blank" rel="noreferrer" class="text-indigo-400 hover:underline truncate">
                        {owner.website}
                      </a>
                    </div>
                  )}
                </div>
              </div>

              <div class="flex items-center justify-between pt-4 mt-5 border-t border-slate-800">
                <span class="text-xs text-slate-400">
                  <strong class="text-slate-100 font-mono text-sm">{owner.projects_count}</strong> projects
                </span>

                <div class="flex items-center gap-2">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleToggleActive(owner.id)}
                    class={owner.active ? "text-amber-400" : "text-emerald-400"}
                  >
                    {owner.active ? "Deactivate" : "Activate"}
                  </Button>
                  <A href={`/owners/${owner.id}`}>
                    <Button variant="secondary" size="sm">
                      Profile <ExternalLink class="w-3 h-3 ml-1" />
                    </Button>
                  </A>
                </div>
              </div>
            </div>
          )}
        </For>
      </div>

      <Show when={owners().length === 0 && !loading()}>
        <div class="text-center py-16 bg-slate-900/40 border border-slate-800 rounded-2xl">
          <Users class="w-10 h-10 mx-auto text-slate-600 mb-2" />
          <p class="text-sm font-medium text-slate-300">No project owners found</p>
          <p class="text-xs text-slate-500 mt-1">Register a partner developer to get started.</p>
        </div>
      </Show>

      {/* Register Owner Modal */}
      <Modal
        isOpen={modalOpen()}
        onClose={() => setModalOpen(false)}
        title="Register Project Owner"
        description="Creates credentials and a partner organization account for a builder or developer."
        footer={
          <>
            <Button variant="ghost" onClick={() => setModalOpen(false)}>
              Cancel
            </Button>
            <Button loading={submitting()} onClick={handleCreateOwner}>
              Register Partner
            </Button>
          </>
        }
      >
        <form onSubmit={handleCreateOwner} class="space-y-4">
          {formError() && (
            <div class="p-3 bg-rose-950/60 border border-rose-800 rounded-lg text-xs text-rose-300">
              {formError()}
            </div>
          )}

          <Input
            label="Organization / Company Name"
            required
            placeholder="e.g. Prestige Estates Ltd"
            value={formName()}
            onInput={(e) => setFormName(e.currentTarget.value)}
          />

          <div class="grid grid-cols-2 gap-4">
            <Input
              label="Login Username"
              required
              placeholder="e.g. prestige_admin"
              value={formUsername()}
              onInput={(e) => setFormUsername(e.currentTarget.value)}
            />
            <Input
              label="Initial Password"
              type="password"
              required
              placeholder="••••••••••••"
              value={formPassword()}
              onInput={(e) => setFormPassword(e.currentTarget.value)}
            />
          </div>

          <div class="grid grid-cols-2 gap-4">
            <Input
              label="Contact Email"
              type="email"
              placeholder="info@prestige.com"
              value={formEmail()}
              onInput={(e) => setFormEmail(e.currentTarget.value)}
            />
            <Input
              label="Phone Number"
              placeholder="+1 (555) 000-0000"
              value={formPhone()}
              onInput={(e) => setFormPhone(e.currentTarget.value)}
            />
          </div>

          <Input
            label="Official Website"
            placeholder="https://prestige.example.com"
            value={formWebsite()}
            onInput={(e) => setFormWebsite(e.currentTarget.value)}
          />

          <Textarea
            label="Corporate Bio / Overview"
            placeholder="About the developer, track record, years in industry..."
            value={formBio()}
            onInput={(e) => setFormBio(e.currentTarget.value)}
          />
        </form>
      </Modal>
    </div>
  );
}

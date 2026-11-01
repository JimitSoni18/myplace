import { createSignal, onMount, For } from "solid-js";
import { ShieldCheck, UserPlus } from "lucide-solid";
import { Button } from "@/shared/components/Button";
import { Input } from "@/shared/components/Input";
import { Badge } from "@/shared/components/Badge";
import { Modal } from "@/shared/components/Modal";
import { api } from "@/shared/api/client";
import { AdminUserItem, CreateAdminUserPayload } from "@/shared/api/types";

export function AdminUsers() {
  const [users, setUsers] = createSignal<AdminUserItem[]>([]);

  // Modal
  const [modalOpen, setModalOpen] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const [formUsername, setFormUsername] = createSignal("");
  const [formPassword, setFormPassword] = createSignal("");
  const [formConfirmPassword, setFormConfirmPassword] = createSignal("");

  const fetchUsers = async () => {
    try {
      const res = await api.get<AdminUserItem[]>("/api/admin/users");
      setUsers(res);
    } catch (e) {
      console.error(e);
    }
  };

  onMount(fetchUsers);

  const handleCreate = async (e: Event) => {
    e.preventDefault();
    if (!formUsername().trim() || !formPassword() || !formConfirmPassword()) {
      setError("Username, password, and confirmation are required.");
      return;
    }

    if (formPassword() !== formConfirmPassword()) {
      setError("Passwords do not match.");
      return;
    }

    setSubmitting(true);
    setError(null);

    try {
      const payload: CreateAdminUserPayload = {
        username: formUsername().trim(),
        password: formPassword(),
        confirm_password: formConfirmPassword(),
      };

      await api.post("/api/admin/users", payload);

      setModalOpen(false);
      setFormUsername("");
      setFormPassword("");
      setFormConfirmPassword("");
      fetchUsers();
    } catch (err: any) {
      setError(err.message || "Failed to create admin user");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div class="space-y-6">
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 pb-4 border-b border-slate-800">
        <div>
          <h1 class="text-2xl font-bold tracking-tight text-white">System Administrative Staff</h1>
          <p class="text-xs text-slate-400 mt-1">
            Authorized admin user profiles with access to the console and JSON endpoints
          </p>
        </div>

        <Button onClick={() => setModalOpen(true)}>
          <UserPlus class="w-4 h-4" /> Register Staff Member
        </Button>
      </div>

      <div class="bg-slate-900 border border-slate-800 rounded-2xl overflow-hidden shadow-sm">
        <div class="overflow-x-auto">
          <table class="w-full text-left text-xs">
            <thead class="bg-slate-950 text-slate-400 uppercase tracking-wider font-semibold border-b border-slate-800">
              <tr>
                <th class="px-6 py-3.5">Staff Username</th>
                <th class="px-6 py-3.5">Assigned Role</th>
                <th class="px-6 py-3.5">Account Status</th>
                <th class="px-6 py-3.5">Created Date</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-800/60">
              <For each={users()}>
                {(user) => (
                  <tr class="hover:bg-slate-800/40 transition-colors">
                    <td class="px-6 py-4 font-semibold text-slate-100 flex items-center gap-2">
                      <ShieldCheck class="w-3.5 h-3.5 text-indigo-400 shrink-0" />
                      {user.username}
                    </td>
                    <td class="px-6 py-4">
                      <Badge variant="info">Admin</Badge>
                    </td>
                    <td class="px-6 py-4">
                      <Badge variant={user.is_current_user ? "success" : "default"}>
                        {user.is_current_user ? "Current User" : "Active Staff"}
                      </Badge>
                    </td>
                    <td class="px-6 py-4 text-slate-400 font-mono">
                      {new Date(user.created_at).toLocaleDateString()}
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
        title="Register Administrative Staff"
        description="Grants full management rights over projects, properties, and partner owners."
        footer={
          <>
            <Button variant="ghost" onClick={() => setModalOpen(false)}>
              Cancel
            </Button>
            <Button loading={submitting()} onClick={handleCreate}>
              Create Staff Account
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
            label="Staff Username"
            required
            placeholder="e.g. sarah_ops"
            value={formUsername()}
            onInput={(e) => setFormUsername(e.currentTarget.value)}
          />

          <Input
            label="Password"
            type="password"
            required
            placeholder="••••••••••••"
            value={formPassword()}
            onInput={(e) => setFormPassword(e.currentTarget.value)}
          />

          <Input
            label="Confirm Password"
            type="password"
            required
            placeholder="••••••••••••"
            value={formConfirmPassword()}
            onInput={(e) => setFormConfirmPassword(e.currentTarget.value)}
          />
        </form>
      </Modal>
    </div>
  );
}

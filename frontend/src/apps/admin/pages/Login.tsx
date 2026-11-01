import { createSignal } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { ArrowRight } from "lucide-solid";
import { Button } from "@/shared/components/Button";
import { Input } from "@/shared/components/Input";
import { api, setToken, setCurrentUser } from "@/shared/api/client";
import { LoginResponse } from "@/shared/api/types";

export function Login() {
  const navigate = useNavigate();
  const [username, setUsername] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    if (!username() || !password()) {
      setError("Please enter both username and password");
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const res = await api.post<LoginResponse>("/api/auth/admin/login", {
        username: username().trim(),
        password: password(),
      });

      if (res.token) {
        setToken(res.token);
        setCurrentUser(res.user);
        navigate("/");
      } else {
        setError("Invalid response from authentication server");
      }
    } catch (err: any) {
      setError(err.message || "Invalid administrative credentials");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="min-h-screen flex items-center justify-center p-4 bg-slate-950">
      <div class="w-full max-w-md">
        {/* Brand Card Header */}
        <div class="text-center mb-8">
          <div class="inline-flex items-center justify-center w-12 h-12 rounded-2xl bg-gradient-to-tr from-indigo-600 to-indigo-500 shadow-xl shadow-indigo-500/25 text-white font-bold text-xl mb-4">
            M
          </div>
          <h1 class="text-2xl font-bold tracking-tight text-white">MyPlace Admin Portal</h1>
          <p class="text-xs text-slate-400 mt-1.5">Authorized administrative management console</p>
        </div>

        {/* Login Form Box */}
        <div class="bg-slate-900/80 backdrop-blur-xl border border-slate-800 rounded-2xl p-6 sm:p-8 shadow-2xl shadow-black/80">
          <form onSubmit={handleSubmit} class="space-y-4">
            {error() && (
              <div class="p-3 bg-rose-950/60 border border-rose-800 rounded-xl text-xs text-rose-300 font-medium">
                {error()}
              </div>
            )}

            <div>
              <Input
                label="Admin Username"
                type="text"
                required
                autocomplete="username"
                placeholder="e.g. admin"
                value={username()}
                onInput={(e) => setUsername(e.currentTarget.value)}
              />
            </div>

            <div>
              <Input
                label="Password"
                type="password"
                required
                autocomplete="current-password"
                placeholder="••••••••••••"
                value={password()}
                onInput={(e) => setPassword(e.currentTarget.value)}
              />
            </div>

            <div class="pt-2">
              <Button type="submit" loading={loading()} class="w-full">
                Sign in to Console <ArrowRight class="w-4 h-4" />
              </Button>
            </div>
          </form>
        </div>

        <p class="text-center text-[11px] text-slate-600 mt-6">
          Internal administrative system • Unconditional JSON API
        </p>
      </div>
    </div>
  );
}

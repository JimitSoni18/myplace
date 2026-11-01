import { createSignal } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { ArrowRight, Building } from "lucide-solid";
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
      setError("Please provide username and password");
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const res = await api.post<LoginResponse>("/api/auth/owner/login", {
        username: username().trim(),
        password: password(),
      });

      if (res.token) {
        setToken(res.token);
        setCurrentUser(res.user);
        navigate("/");
      } else {
        setError("Invalid response from partner server");
      }
    } catch (err: any) {
      setError(err.message || "Invalid partner developer credentials");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="min-h-screen flex items-center justify-center p-4 bg-slate-950">
      <div class="w-full max-w-md">
        <div class="text-center mb-8">
          <div class="inline-flex items-center justify-center w-12 h-12 rounded-2xl bg-gradient-to-tr from-emerald-600 to-teal-500 shadow-xl shadow-emerald-500/25 text-white font-bold text-xl mb-4">
            <Building class="w-6 h-6" />
          </div>
          <h1 class="text-2xl font-bold tracking-tight text-white">Partner Developer Portal</h1>
          <p class="text-xs text-slate-400 mt-1.5">Authorized property developer management console</p>
        </div>

        <div class="bg-slate-900/80 backdrop-blur-xl border border-slate-800 rounded-2xl p-6 sm:p-8 shadow-2xl shadow-black/80">
          <form onSubmit={handleSubmit} class="space-y-4">
            {error() && (
              <div class="p-3 bg-rose-950/60 border border-rose-800 rounded-xl text-xs text-rose-300 font-medium">
                {error()}
              </div>
            )}

            <div>
              <Input
                label="Partner Username"
                type="text"
                required
                autocomplete="username"
                placeholder="Partner account ID"
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
              <Button type="submit" loading={loading()} class="w-full bg-emerald-600 hover:bg-emerald-500 focus-visible:outline-emerald-500">
                Sign in to Partner Console <ArrowRight class="w-4 h-4" />
              </Button>
            </div>
          </form>
        </div>

        <p class="text-center text-[11px] text-slate-600 mt-6">
          MyPlace Partner Network • Secure Real Estate Partner Console
        </p>
      </div>
    </div>
  );
}

import { JSX, createSignal, For } from "solid-js";
import { A, useLocation, useNavigate } from "@solidjs/router";
import { LayoutDashboard, Building2, LogOut, Menu, X, ShieldCheck } from "lucide-solid";
import { getCurrentUser, clearToken } from "@/shared/api/client";
import { NotificationBell } from "@/shared/components/NotificationBell";

interface OwnerNavProps {
  children: JSX.Element;
}

export function OwnerNav(props: OwnerNavProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const [mobileMenuOpen, setMobileMenuOpen] = createSignal(false);
  const user = getCurrentUser();

  const handleLogout = () => {
    clearToken();
    navigate("/login");
  };

  const navItems = [
    { name: "Partner Dashboard", href: "/", icon: LayoutDashboard },
    { name: "My Projects", href: "/projects", icon: Building2 },
  ];

  const isActive = (href: string) => {
    if (href === "/") return location.pathname === "/";
    return location.pathname.startsWith(href);
  };

  return (
    <div class="h-screen flex overflow-hidden bg-slate-950">
      {/* Sidebar Desktop */}
      <aside class="hidden lg:flex lg:flex-col lg:w-64 h-full border-r border-slate-800/80 bg-slate-900/60 backdrop-blur-xl shrink-0">
        <div class="flex items-center gap-3 px-6 h-16 border-b border-slate-800/80">
          <div class="w-8 h-8 rounded-lg bg-gradient-to-tr from-emerald-600 to-teal-500 flex items-center justify-center font-bold text-white shadow-lg shadow-emerald-500/20">
            P
          </div>
          <div>
            <div class="text-sm font-bold tracking-tight text-white">MyPlace</div>
            <div class="text-[10px] tracking-widest uppercase font-semibold text-emerald-400">
              Partner Portal
            </div>
          </div>
        </div>

        <nav class="flex-1 px-3 py-4 space-y-1 overflow-y-auto">
          <For each={navItems}>
            {(item) => {
              const Icon = item.icon;
              return (
                <A
                  href={item.href}
                  class={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all ${
                    isActive(item.href)
                      ? "bg-emerald-600/15 text-emerald-400 font-semibold border border-emerald-500/20 shadow-sm"
                      : "text-slate-400 hover:text-slate-100 hover:bg-slate-800/60"
                  }`}
                >
                  <Icon class={`w-4 h-4 ${isActive(item.href) ? "text-emerald-400" : "text-slate-400"}`} />
                  {item.name}
                </A>
              );
            }}
          </For>
        </nav>

        <div class="p-3 border-t border-slate-800/80">
          <div class="flex items-center justify-between p-2.5 rounded-lg bg-slate-900 border border-slate-800">
            <div class="truncate mr-2">
              <p class="text-xs font-semibold text-slate-200 truncate">{user?.owner_name || user?.username || "Partner"}</p>
              <p class="text-[10px] font-medium text-emerald-400 tracking-wider uppercase">Verified Developer</p>
            </div>
            <button
              onClick={handleLogout}
              title="Sign out"
              class="p-1.5 text-slate-400 hover:text-rose-400 hover:bg-slate-800 rounded-md transition-colors cursor-pointer"
            >
              <LogOut class="w-4 h-4" />
            </button>
          </div>
        </div>
      </aside>

      {/* Main Content Area */}
      <div class="flex-1 flex flex-col h-full min-w-0 overflow-hidden">
        <header class="h-16 border-b border-slate-800/80 bg-slate-900/40 backdrop-blur-md px-4 sm:px-6 flex items-center justify-between gap-4 shrink-0 z-30">
          <div class="flex items-center gap-3">
            <button
              onClick={() => setMobileMenuOpen(!mobileMenuOpen())}
              class="lg:hidden p-2 text-slate-400 hover:text-white rounded-lg hover:bg-slate-800"
            >
              {mobileMenuOpen() ? <X class="w-5 h-5" /> : <Menu class="w-5 h-5" />}
            </button>
            <div class="text-xs font-semibold text-slate-300">
              Developer Partner Console
            </div>
          </div>

          <div class="flex items-center gap-3">
            <NotificationBell endpointPrefix="/api/owner/notifications" />
            <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-emerald-950/60 text-emerald-400 border border-emerald-800/60">
              <ShieldCheck class="w-3.5 h-3.5" />
              Developer Partner
            </span>
          </div>

        </header>

        {mobileMenuOpen() && (
          <div class="lg:hidden border-b border-slate-800 bg-slate-900 px-4 py-3 space-y-1">
            <For each={navItems}>
              {(item) => {
                const Icon = item.icon;
                return (
                  <A
                    href={item.href}
                    onClick={() => setMobileMenuOpen(false)}
                    class={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium ${
                      isActive(item.href) ? "bg-emerald-600 text-white font-semibold" : "text-slate-300 hover:bg-slate-800"
                    }`}
                  >
                    <Icon class="w-4 h-4" />
                    {item.name}
                  </A>
                );
              }}
            </For>
            <button
              onClick={handleLogout}
              class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium text-rose-400 hover:bg-slate-800 text-left mt-2 border-t border-slate-800"
            >
              <LogOut class="w-4 h-4" />
              Sign out
            </button>
          </div>
        )}

        <main class="flex-1 p-4 sm:p-6 lg:p-8 overflow-y-auto max-w-7xl w-full mx-auto">
          {props.children}
        </main>
      </div>
    </div>
  );
}

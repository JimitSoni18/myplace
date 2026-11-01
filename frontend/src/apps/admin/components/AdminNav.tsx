import { JSX, createSignal, For } from "solid-js";
import { A, useLocation, useNavigate } from "@solidjs/router";
import {
  LayoutDashboard,
  Building2,
  Users,
  Sliders,
  LogOut,
  Search,
  Menu,
  X,
} from "lucide-solid";
import { getCurrentUser, clearToken } from "@/shared/api/client";
import { NotificationBell } from "@/shared/components/NotificationBell";

interface AdminNavProps {
  onOpenCommandPalette: () => void;
  children: JSX.Element;
}

export function AdminNav(props: AdminNavProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const [mobileMenuOpen, setMobileMenuOpen] = createSignal(false);
  const user = getCurrentUser();

  const handleLogout = () => {
    clearToken();
    navigate("/login");
  };

  const navItems = [
    { name: "Dashboard", href: "/", icon: LayoutDashboard },
    { name: "Projects", href: "/projects", icon: Building2 },
    { name: "Developers", href: "/owners", icon: Users },
    { name: "Platform Settings", href: "/settings", icon: Sliders },
  ];

  const isActive = (href: string) => {
    if (href === "/") return location.pathname === "/";
    return location.pathname.startsWith(href);
  };

  return (
    <div class="h-screen flex overflow-hidden bg-slate-950">
      {/* Sidebar Desktop */}
      <aside class="hidden lg:flex lg:flex-col lg:w-64 h-full border-r border-slate-800/80 bg-slate-900/60 backdrop-blur-xl shrink-0">
        {/* Logo */}
        <div class="flex items-center gap-3 px-6 h-16 border-b border-slate-800/80">
          <div class="w-8 h-8 rounded-lg bg-gradient-to-tr from-indigo-600 to-indigo-400 flex items-center justify-center font-bold text-white shadow-lg shadow-indigo-500/20">
            M
          </div>
          <div>
            <div class="text-sm font-bold tracking-tight text-white">MyPlace</div>
            <div class="text-[10px] tracking-widest uppercase font-semibold text-indigo-400">
              Admin Portal
            </div>
          </div>
        </div>

        {/* Navigation items */}
        <nav class="flex-1 px-3 py-4 space-y-1 overflow-y-auto">
          <For each={navItems}>
            {(item) => {
              const Icon = item.icon;
              return (
                <A
                  href={item.href}
                  class={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all ${
                    isActive(item.href)
                      ? "bg-indigo-600/15 text-indigo-400 font-semibold border border-indigo-500/20 shadow-sm"
                      : "text-slate-400 hover:text-slate-100 hover:bg-slate-800/60"
                  }`}
                >
                  <Icon class={`w-4 h-4 ${isActive(item.href) ? "text-indigo-400" : "text-slate-400"}`} />
                  {item.name}
                </A>
              );
            }}
          </For>
        </nav>

        {/* User footer */}
        <div class="p-3 border-t border-slate-800/80">
          <div class="flex items-center justify-between p-2.5 rounded-lg bg-slate-900 border border-slate-800">
            <div class="truncate mr-2">
              <p class="text-xs font-semibold text-slate-200 truncate">{user?.username || "Admin"}</p>
              <p class="text-[10px] font-medium text-indigo-400 tracking-wider uppercase">System Staff</p>
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
        {/* Top Header */}
        <header class="h-16 border-b border-slate-800/80 bg-slate-900/40 backdrop-blur-md px-4 sm:px-6 flex items-center justify-between gap-4 shrink-0 z-30">
          <div class="flex items-center gap-3">
            {/* Mobile Menu button */}
            <button
              onClick={() => setMobileMenuOpen(!mobileMenuOpen())}
              class="lg:hidden p-2 text-slate-400 hover:text-white rounded-lg hover:bg-slate-800"
            >
              {mobileMenuOpen() ? <X class="w-5 h-5" /> : <Menu class="w-5 h-5" />}
            </button>

            {/* Quick search button (Cmd+K) */}
            <button
              onClick={props.onOpenCommandPalette}
              class="flex items-center gap-2.5 px-3.5 py-1.5 bg-slate-900 hover:bg-slate-850 border border-slate-800 hover:border-slate-700 rounded-lg text-slate-400 text-xs transition-colors w-48 sm:w-72 cursor-pointer text-left"
            >
              <Search class="w-3.5 h-3.5 text-slate-500 shrink-0" />
              <span class="truncate">Search projects, units, owners...</span>
              <kbd class="ml-auto hidden sm:inline-block px-1.5 py-0.5 text-[10px] font-mono bg-slate-800 border border-slate-700 rounded text-slate-400">
                ⌘K
              </kbd>
            </button>
          </div>

          <div class="flex items-center gap-3">
            <NotificationBell endpointPrefix="/api/admin/notifications" />
            <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-emerald-950/60 text-emerald-400 border border-emerald-800/60">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
              Live API
            </span>
          </div>

        </header>

        {/* Mobile Navigation Drawer */}
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
                      isActive(item.href) ? "bg-indigo-600 text-white font-semibold" : "text-slate-300 hover:bg-slate-800"
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

        {/* Page Content */}
        <main class="flex-1 p-4 sm:p-6 lg:p-8 overflow-y-auto max-w-7xl w-full mx-auto">
          {props.children}
        </main>
      </div>
    </div>
  );
}

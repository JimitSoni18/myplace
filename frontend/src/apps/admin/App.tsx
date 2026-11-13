import { createSignal, onMount, onCleanup } from "solid-js";
import { HashRouter, Route, useNavigate, type RouteSectionProps } from "@solidjs/router";
import { AdminNav } from "./components/AdminNav";
import { CommandPalette } from "./components/CommandPalette";
import { Login } from "./pages/Login";
import { Dashboard } from "./pages/Dashboard";
import { ProjectsList } from "./pages/ProjectsList";
import { ProjectDetail } from "./pages/ProjectDetail";
import { OwnersList } from "./pages/OwnersList";
import { OwnerDetail } from "./pages/OwnerDetail";
import { getToken } from "@/shared/api/client";
import { Settings } from "./pages/Settings";

export function App() {
  const [paletteOpen, setPaletteOpen] = createSignal(false);

  // Global Cmd+K / Ctrl+K listener
  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setPaletteOpen((prev) => !prev);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    onCleanup(() => window.removeEventListener("keydown", handleKeyDown));
  });

  // Persistent Auth Guard layout: mounted once, only children reconcile
  const ProtectedLayout = (props: RouteSectionProps) => {
    const navigate = useNavigate();
    const token = getToken();

    if (!token) {
      navigate("/login", { replace: true });
      return null;
    }

    return (
      <AdminNav onOpenCommandPalette={() => setPaletteOpen(true)}>
        {props.children}
        <CommandPalette isOpen={paletteOpen()} onClose={() => setPaletteOpen(false)} />
      </AdminNav>
    );
  };

  return (
    <HashRouter>
      <Route path="/login" component={Login} />
      <Route path="/" component={ProtectedLayout}>
        <Route path="/" component={Dashboard} />
        <Route path="/projects" component={ProjectsList} />
        <Route path="/projects/:id" component={ProjectDetail} />
        <Route path="/owners" component={OwnersList} />
        <Route path="/owners/:id" component={OwnerDetail} />
        <Route path="/settings" component={Settings} />
        <Route path="/amenities" component={Settings} />
        <Route path="/users" component={Settings} />
        <Route path="*param" component={Dashboard} />
      </Route>
    </HashRouter>
  );
}

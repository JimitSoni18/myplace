import { HashRouter, Route, useNavigate, type RouteSectionProps } from "@solidjs/router";
import { OwnerNav } from "./components/OwnerNav";
import { Login } from "./pages/Login";
import { Dashboard } from "./pages/Dashboard";
import { ProjectsList } from "./pages/ProjectsList";
import { ProjectDetail } from "./pages/ProjectDetail";
import { getToken, getCurrentUser } from "@/shared/api/client";

export function App() {
  // Persistent Owner Auth Guard layout
  const ProtectedLayout = (props: RouteSectionProps) => {
    const navigate = useNavigate();
    const token = getToken();
    const user = getCurrentUser();

    if (!token || user?.role !== "owner") {
      navigate("/login", { replace: true });
      return null;
    }

    return (
      <OwnerNav>
        {props.children}
      </OwnerNav>
    );
  };

  return (
    <HashRouter>
      <Route path="/login" component={Login} />
      <Route path="/" component={ProtectedLayout}>
        <Route path="/" component={Dashboard} />
        <Route path="/projects" component={ProjectsList} />
        <Route path="/projects/:id" component={ProjectDetail} />
      </Route>
    </HashRouter>
  );
}

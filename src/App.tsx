import { useStore } from "./store/useStore";
import { useDb } from "./hooks/useDb";
import { lazy, Suspense, useEffect } from "react";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { Layout } from "./components/Layout";
import { AppLoader } from "./components/Skeleton";

const Dashboard = lazy(() => import("./pages/Dashboard").then(m => ({ default: m.Dashboard })));
const Sessions = lazy(() => import("./pages/Sessions").then(m => ({ default: m.Sessions })));
const Analytics = lazy(() => import("./pages/Analytics").then(m => ({ default: m.Analytics })));
const Projects = lazy(() => import("./pages/Projects").then(m => ({ default: m.Projects })));
const Settings = lazy(() => import("./pages/Settings").then(m => ({ default: m.Settings })));

function App() {
  const currentTab = useStore((s) => s.currentTab);
  const { refreshAll } = useDb();

  useEffect(() => {
    refreshAll();
  }, [refreshAll]);

  const pages: Record<string, React.ReactNode> = {
    dashboard: <Dashboard />,
    sessions: <Sessions />,
    analytics: <Analytics />,
    projects: <Projects />,
    settings: <Settings />,
  };

  return (
    <ErrorBoundary>
      <Suspense fallback={<AppLoader />}>
        <Layout>{pages[currentTab] ?? <Dashboard />}</Layout>
      </Suspense>
    </ErrorBoundary>
  );
}

export default App;

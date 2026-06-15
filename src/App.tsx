import { useStore } from "./store/useStore";
import { useDb } from "./hooks/useDb";
import { useEffect } from "react";
import { Layout } from "./components/Layout";
import { Dashboard } from "./pages/Dashboard";
import { Sessions } from "./pages/Sessions";
import { Analytics } from "./pages/Analytics";
import { Projects } from "./pages/Projects";
import { Settings } from "./pages/Settings";

function App() {
  const currentTab = useStore((s) => s.currentTab);
  const { refreshAll } = useDb();

  useEffect(() => {
    refreshAll();
  }, []);

  const pages: Record<string, React.ReactNode> = {
    dashboard: <Dashboard />,
    sessions: <Sessions />,
    analytics: <Analytics />,
    projects: <Projects />,
    settings: <Settings />,
  };

  return <Layout>{pages[currentTab] ?? <Dashboard />}</Layout>;
}

export default App;

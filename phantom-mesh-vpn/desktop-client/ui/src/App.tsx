import { useEffect } from "react";
import { Routes, Route, useNavigate } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { useVpnStore } from "./stores/vpnStore";
import Layout from "./components/Layout";
import Dashboard from "./pages/Dashboard";
import Servers from "./pages/Servers";
import Settings from "./pages/Settings";
import Account from "./pages/Account";
import About from "./pages/About";

function App() {
  const navigate = useNavigate();
  const { connect, disconnect, quickConnect } = useVpnStore();

  useEffect(() => {
    // Listen for tray events
    const unlistenNavigate = listen<string>("navigate", (event) => {
      navigate(event.payload);
    });

    const unlistenQuickConnect = listen("tray-quick-connect", async () => {
      try {
        await quickConnect();
      } catch {
        // Error handled by store
      }
    });

    const unlistenDisconnect = listen("tray-disconnect", async () => {
      try {
        await disconnect();
      } catch {
        // Error handled by store
      }
    });

    const unlistenConnect = listen<string>("tray-connect", async (event) => {
      try {
        await connect(event.payload);
      } catch {
        // Error handled by store
      }
    });

    return () => {
      unlistenNavigate.then((fn) => fn());
      unlistenQuickConnect.then((fn) => fn());
      unlistenDisconnect.then((fn) => fn());
      unlistenConnect.then((fn) => fn());
    };
  }, [navigate, connect, disconnect, quickConnect]);

  return (
    <Layout>
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/servers" element={<Servers />} />
        <Route path="/settings" element={<Settings />} />
        <Route path="/account" element={<Account />} />
        <Route path="/about" element={<About />} />
      </Routes>
    </Layout>
  );
}

export default App;

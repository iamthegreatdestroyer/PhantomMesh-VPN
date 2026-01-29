import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface AppInfo {
  version: string;
  name: string;
  tauriVersion: string;
}

const DEFAULT_APP_INFO: AppInfo = {
  version: "1.0.0",
  name: "PhantomMesh VPN",
  tauriVersion: "2.0.0",
};

export function useAppInfo() {
  const [appInfo, setAppInfo] = useState<AppInfo>(DEFAULT_APP_INFO);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchInfo() {
      try {
        // Backend only provides version string via get_app_version
        const version = await invoke<string>("get_app_version");
        setAppInfo({
          ...DEFAULT_APP_INFO,
          version,
        });
      } catch {
        // Use defaults on error
      } finally {
        setLoading(false);
      }
    }

    fetchInfo();
  }, []);

  return { appInfo, loading };
}

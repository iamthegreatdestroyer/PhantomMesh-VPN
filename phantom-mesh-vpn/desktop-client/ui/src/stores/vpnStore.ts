import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export type ConnectionState =
  | "disconnected"
  | "connecting"
  | "connected"
  | "disconnecting"
  | "reconnecting"
  | "error";

export type VpnProtocol = "wireguard" | "openvpn" | "stealth";

export interface ServerInfo {
  id: string;
  name: string;
  country: string;
  countryCode: string;
  city: string;
  load: number;
  latency?: number;
  features: string[];
  ipAddress: string;
  protocol: VpnProtocol;
}

export interface ConnectionStats {
  bytesIn: number;
  bytesOut: number;
  duration: number;
  currentSpeed: {
    download: number;
    upload: number;
  };
}

export interface VpnSettings {
  killSwitch: boolean;
  autoConnect: boolean;
  protocol: VpnProtocol;
  splitTunneling: boolean;
  excludedApps: string[];
  dnsServers: string[];
  startMinimized: boolean;
  showNotifications: boolean;
}

interface VpnState {
  // Connection state
  connectionState: ConnectionState;
  currentServer: ServerInfo | null;
  connectionStats: ConnectionStats | null;

  // Server list
  servers: ServerInfo[];
  favoriteServers: string[];
  recentServers: string[];

  // Settings
  settings: VpnSettings;

  // Actions
  connect: (serverId: string) => Promise<void>;
  disconnect: () => Promise<void>;
  quickConnect: () => Promise<void>;

  // Server actions
  fetchServers: () => Promise<void>;
  toggleFavorite: (serverId: string) => void;

  // Settings actions
  updateSettings: (settings: Partial<VpnSettings>) => Promise<void>;
  toggleKillSwitch: () => Promise<void>;
  toggleAutoConnect: () => Promise<void>;

  // Stats
  fetchStats: () => Promise<void>;
}

const defaultSettings: VpnSettings = {
  killSwitch: true,
  autoConnect: false,
  protocol: "wireguard",
  splitTunneling: false,
  excludedApps: [],
  dnsServers: ["1.1.1.1", "1.0.0.1"],
  startMinimized: false,
  showNotifications: true,
};

export const useVpnStore = create<VpnState>((set, get) => ({
  // Initial state
  connectionState: "disconnected",
  currentServer: null,
  connectionStats: null,
  servers: [],
  favoriteServers: [],
  recentServers: [],
  settings: defaultSettings,

  // Connect to a server
  connect: async (serverId: string) => {
    set({ connectionState: "connecting" });

    try {
      const result = await invoke<{ server: ServerInfo }>("connect", {
        serverId,
      });

      set({
        connectionState: "connected",
        currentServer: result.server,
        recentServers: [
          serverId,
          ...get().recentServers.filter((id) => id !== serverId),
        ].slice(0, 5),
      });

      // Start stats polling
      get().fetchStats();
    } catch (error) {
      console.error("Connection failed:", error);
      set({ connectionState: "error" });
      throw error;
    }
  },

  // Disconnect
  disconnect: async () => {
    set({ connectionState: "disconnecting" });

    try {
      await invoke("disconnect");
      set({
        connectionState: "disconnected",
        currentServer: null,
        connectionStats: null,
      });
    } catch (error) {
      console.error("Disconnect failed:", error);
      set({ connectionState: "error" });
      throw error;
    }
  },

  // Quick connect to best server
  quickConnect: async () => {
    set({ connectionState: "connecting" });

    try {
      const result = await invoke<{ server: ServerInfo }>("quick_connect");

      set({
        connectionState: "connected",
        currentServer: result.server,
        recentServers: [
          result.server.id,
          ...get().recentServers.filter((id) => id !== result.server.id),
        ].slice(0, 5),
      });

      get().fetchStats();
    } catch (error) {
      console.error("Quick connect failed:", error);
      set({ connectionState: "error" });
      throw error;
    }
  },

  // Fetch server list
  fetchServers: async () => {
    try {
      const servers = await invoke<ServerInfo[]>("get_servers");
      set({ servers });
    } catch (error) {
      console.error("Failed to fetch servers:", error);
    }
  },

  // Toggle favorite server
  toggleFavorite: (serverId: string) => {
    const { favoriteServers } = get();

    if (favoriteServers.includes(serverId)) {
      set({ favoriteServers: favoriteServers.filter((id) => id !== serverId) });
    } else {
      set({ favoriteServers: [...favoriteServers, serverId] });
    }
  },

  // Update settings
  updateSettings: async (newSettings: Partial<VpnSettings>) => {
    const updatedSettings = { ...get().settings, ...newSettings };

    try {
      await invoke("update_settings", { settings: updatedSettings });
      set({ settings: updatedSettings });
    } catch (error) {
      console.error("Failed to update settings:", error);
      throw error;
    }
  },

  // Toggle kill switch
  toggleKillSwitch: async () => {
    const enabled = !get().settings.killSwitch;

    try {
      await invoke("toggle_kill_switch", { enabled });
      set({ settings: { ...get().settings, killSwitch: enabled } });
    } catch (error) {
      console.error("Failed to toggle kill switch:", error);
      throw error;
    }
  },

  // Toggle auto connect
  toggleAutoConnect: async () => {
    const enabled = !get().settings.autoConnect;

    try {
      await invoke("toggle_auto_connect", { enabled });
      set({ settings: { ...get().settings, autoConnect: enabled } });
    } catch (error) {
      console.error("Failed to toggle auto connect:", error);
      throw error;
    }
  },

  // Fetch connection stats
  fetchStats: async () => {
    if (get().connectionState !== "connected") return;

    try {
      const stats = await invoke<ConnectionStats>("get_connection_stats");
      set({ connectionStats: stats });
    } catch (error) {
      console.error("Failed to fetch stats:", error);
    }
  },
}));

import { motion } from "framer-motion";
import {
  Shield,
  Zap,
  Globe,
  Lock,
  Bell,
  Monitor,
  Network,
  Wifi,
  Eye,
  EyeOff,
  ChevronRight,
} from "lucide-react";
import clsx from "clsx";
import { useVpnStore, VpnProtocol } from "../stores/vpnStore";

interface SettingToggleProps {
  label: string;
  description?: string;
  icon: typeof Shield;
  enabled: boolean;
  onToggle: () => void;
  disabled?: boolean;
}

function SettingToggle({
  label,
  description,
  icon: Icon,
  enabled,
  onToggle,
  disabled,
}: SettingToggleProps) {
  return (
    <button
      onClick={onToggle}
      disabled={disabled}
      className={clsx(
        "w-full flex items-center gap-4 p-4 rounded-xl transition-all",
        "hover:bg-white/5",
        disabled && "opacity-50 cursor-not-allowed",
      )}
    >
      <div
        className={clsx(
          "w-10 h-10 rounded-lg flex items-center justify-center",
          enabled ? "bg-phantom-500/20" : "bg-white/5",
        )}
      >
        <Icon
          className={clsx(
            "w-5 h-5",
            enabled ? "text-phantom-400" : "text-white/60",
          )}
        />
      </div>

      <div className="flex-1 text-left">
        <p className="font-medium text-white">{label}</p>
        {description && (
          <p className="text-xs text-white/50 mt-0.5">{description}</p>
        )}
      </div>

      {/* Toggle switch */}
      <div
        className={clsx(
          "w-12 h-7 rounded-full p-1 transition-colors",
          enabled ? "bg-phantom-500" : "bg-white/20",
        )}
      >
        <motion.div
          className="w-5 h-5 bg-white rounded-full shadow-lg"
          animate={{ x: enabled ? 20 : 0 }}
          transition={{ type: "spring", stiffness: 500, damping: 30 }}
        />
      </div>
    </button>
  );
}

interface SettingSelectProps {
  label: string;
  description?: string;
  icon: typeof Shield;
  value: string;
  options: { value: string; label: string }[];
  onChange: (value: string) => void;
}

function SettingSelect({
  label,
  description,
  icon: Icon,
  value,
  options,
  onChange,
}: SettingSelectProps) {
  return (
    <div className="flex items-center gap-4 p-4 rounded-xl">
      <div className="w-10 h-10 rounded-lg flex items-center justify-center bg-white/5">
        <Icon className="w-5 h-5 text-white/60" />
      </div>

      <div className="flex-1">
        <p className="font-medium text-white">{label}</p>
        {description && (
          <p className="text-xs text-white/50 mt-0.5">{description}</p>
        )}
      </div>

      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="bg-white/10 border border-white/10 rounded-lg px-3 py-2 text-sm
                   text-white focus:outline-none focus:border-phantom-500/50"
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value} className="bg-mesh-dark">
            {opt.label}
          </option>
        ))}
      </select>
    </div>
  );
}

export default function Settings() {
  const {
    settings,
    connectionState,
    toggleKillSwitch,
    toggleAutoConnect,
    updateSettings,
  } = useVpnStore();

  const isConnected = connectionState === "connected";

  const protocolOptions = [
    { value: "wireguard", label: "WireGuard (Recommended)" },
    { value: "openvpn", label: "OpenVPN" },
    { value: "stealth", label: "Stealth (Obfuscated)" },
  ];

  // Wrapper handlers for async functions with proper error handling
  const handleToggleKillSwitch = async () => {
    try {
      await toggleKillSwitch();
    } catch {
      // Error logged in store
    }
  };

  const handleToggleAutoConnect = async () => {
    try {
      await toggleAutoConnect();
    } catch {
      // Error logged in store
    }
  };

  const handleUpdateSettings = async (partial: Parameters<typeof updateSettings>[0]) => {
    try {
      await updateSettings(partial);
    } catch {
      // Error logged in store
    }
  };

  return (
    <div className="space-y-6 overflow-auto h-full pb-4">
      {/* Security Section */}
      <section>
        <h2 className="text-sm font-semibold text-white/40 uppercase tracking-wider mb-3 px-4">
          Security
        </h2>
        <div className="glass rounded-2xl overflow-hidden divide-y divide-white/5">
          <SettingToggle
            label="Kill Switch"
            description="Block internet if VPN disconnects unexpectedly"
            icon={Lock}
            enabled={settings.killSwitch}
            onToggle={handleToggleKillSwitch}
          />
          <SettingToggle
            label="Auto Connect"
            description="Connect automatically when app starts"
            icon={Zap}
            enabled={settings.autoConnect}
            onToggle={handleToggleAutoConnect}
          />
          <SettingSelect
            label="Protocol"
            description="Choose your VPN protocol"
            icon={Shield}
            value={settings.protocol}
            options={protocolOptions}
            onChange={(v) => handleUpdateSettings({ protocol: v as VpnProtocol })}
          />
        </div>
      </section>

      {/* Network Section */}
      <section>
        <h2 className="text-sm font-semibold text-white/40 uppercase tracking-wider mb-3 px-4">
          Network
        </h2>
        <div className="glass rounded-2xl overflow-hidden divide-y divide-white/5">
          <SettingToggle
            label="Split Tunneling"
            description="Exclude certain apps from VPN"
            icon={Network}
            enabled={settings.splitTunneling}
            onToggle={() =>
              handleUpdateSettings({ splitTunneling: !settings.splitTunneling })
            }
          />
          <button className="w-full flex items-center gap-4 p-4 hover:bg-white/5 transition-colors">
            <div className="w-10 h-10 rounded-lg flex items-center justify-center bg-white/5">
              <Globe className="w-5 h-5 text-white/60" />
            </div>
            <div className="flex-1 text-left">
              <p className="font-medium text-white">Custom DNS</p>
              <p className="text-xs text-white/50 mt-0.5">
                {settings.dnsServers.join(", ")}
              </p>
            </div>
            <ChevronRight className="w-5 h-5 text-white/30" />
          </button>
        </div>
      </section>

      {/* Application Section */}
      <section>
        <h2 className="text-sm font-semibold text-white/40 uppercase tracking-wider mb-3 px-4">
          Application
        </h2>
        <div className="glass rounded-2xl overflow-hidden divide-y divide-white/5">
          <SettingToggle
            label="Start Minimized"
            description="Launch app in system tray"
            icon={Monitor}
            enabled={settings.startMinimized}
            onToggle={() =>
              handleUpdateSettings({ startMinimized: !settings.startMinimized })
            }
          />
          <SettingToggle
            label="Notifications"
            description="Show connection status notifications"
            icon={Bell}
            enabled={settings.showNotifications}
            onToggle={() =>
              handleUpdateSettings({ showNotifications: !settings.showNotifications })
            }
          />
        </div>
      </section>

      {/* Connection Status */}
      {isConnected && (
        <motion.div
          className="glass rounded-2xl p-4"
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
        >
          <div className="flex items-center gap-2 text-emerald-400">
            <Wifi className="w-4 h-4" />
            <span className="text-sm font-medium">VPN Active</span>
          </div>
          <p className="text-xs text-white/50 mt-1">
            Some settings cannot be changed while connected
          </p>
        </motion.div>
      )}
    </div>
  );
}

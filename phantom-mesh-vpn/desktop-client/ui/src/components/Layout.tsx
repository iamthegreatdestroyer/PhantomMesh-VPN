import { ReactNode } from "react";
import { NavLink, useLocation } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import { Shield, Globe, Settings, User, Info, X, Minus } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import clsx from "clsx";
import { useVpnStore } from "../stores/vpnStore";

interface LayoutProps {
  children: ReactNode;
}

const navItems = [
  { path: "/", icon: Shield, label: "Connection" },
  { path: "/servers", icon: Globe, label: "Servers" },
  { path: "/settings", icon: Settings, label: "Settings" },
  { path: "/account", icon: User, label: "Account" },
  { path: "/about", icon: Info, label: "About" },
];

export default function Layout({ children }: LayoutProps) {
  const location = useLocation();
  const { connectionState, currentServer } = useVpnStore();
  const appWindow = getCurrentWindow();

  const handleMinimize = () => appWindow.minimize();
  const handleClose = () => appWindow.hide(); // Hide to tray instead of close

  const getStatusColor = () => {
    switch (connectionState) {
      case "connected":
        return "bg-emerald-500";
      case "connecting":
      case "reconnecting":
        return "bg-amber-500";
      case "disconnecting":
        return "bg-amber-500";
      case "error":
        return "bg-red-500";
      default:
        return "bg-gray-500";
    }
  };

  const getStatusText = () => {
    switch (connectionState) {
      case "connected":
        return `Connected to ${currentServer?.city || "Server"}`;
      case "connecting":
        return "Connecting...";
      case "reconnecting":
        return "Reconnecting...";
      case "disconnecting":
        return "Disconnecting...";
      case "error":
        return "Connection Error";
      default:
        return "Not Connected";
    }
  };

  return (
    <div className="h-full flex flex-col mesh-bg">
      {/* Title Bar */}
      <div className="title-bar flex items-center justify-between px-4 py-2 bg-mesh-darker/50 border-b border-white/5">
        <div className="flex items-center gap-3">
          <Shield className="w-5 h-5 text-phantom-500" />
          <span className="text-sm font-medium text-white/80">
            PhantomMesh VPN
          </span>
        </div>

        {/* Status indicator */}
        <div className="flex items-center gap-2">
          <div
            className={clsx("w-2 h-2 rounded-full", getStatusColor(), {
              "animate-pulse":
                connectionState === "connecting" ||
                connectionState === "reconnecting",
            })}
          />
          <span className="text-xs text-white/60">{getStatusText()}</span>
        </div>

        {/* Window controls */}
        <div className="flex items-center gap-1">
          <button
            onClick={handleMinimize}
            className="p-1.5 hover:bg-white/10 rounded transition-colors"
          >
            <Minus className="w-4 h-4 text-white/60" />
          </button>
          <button
            onClick={handleClose}
            className="p-1.5 hover:bg-red-500/80 rounded transition-colors group"
          >
            <X className="w-4 h-4 text-white/60 group-hover:text-white" />
          </button>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-hidden flex flex-col">
        <AnimatePresence mode="wait">
          <motion.main
            key={location.pathname}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.15 }}
            className="flex-1 overflow-auto p-4"
          >
            {children}
          </motion.main>
        </AnimatePresence>

        {/* Navigation */}
        <nav className="flex justify-around py-2 px-4 bg-mesh-darker/50 border-t border-white/5">
          {navItems.map(({ path, icon: Icon, label }) => (
            <NavLink
              key={path}
              to={path}
              className={({ isActive }) =>
                clsx(
                  "flex flex-col items-center gap-1 px-3 py-1.5 rounded-lg transition-all duration-200",
                  isActive
                    ? "text-phantom-400 bg-phantom-500/10"
                    : "text-white/40 hover:text-white/70 hover:bg-white/5",
                )
              }
            >
              <Icon className="w-5 h-5" />
              <span className="text-[10px] font-medium">{label}</span>
            </NavLink>
          ))}
        </nav>
      </div>
    </div>
  );
}

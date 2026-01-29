import { useEffect } from "react";
import { motion } from "framer-motion";
import {
  Shield,
  ShieldOff,
  Zap,
  Globe,
  ArrowDown,
  ArrowUp,
  Clock,
  Lock,
  Unlock,
} from "lucide-react";
import clsx from "clsx";
import { useVpnStore } from "../stores/vpnStore";
import { formatBytes, formatDuration } from "../utils/format";

export default function Dashboard() {
  const {
    connectionState,
    currentServer,
    connectionStats,
    disconnect,
    quickConnect,
    fetchStats,
    settings,
  } = useVpnStore();

  const isConnected = connectionState === "connected";
  const isConnecting =
    connectionState === "connecting" || connectionState === "reconnecting";
  const isDisconnecting = connectionState === "disconnecting";

  // Poll stats when connected
  useEffect(() => {
    if (!isConnected) return;

    const interval = setInterval(fetchStats, 1000);
    return () => clearInterval(interval);
  }, [isConnected, fetchStats]);

  const handleMainButton = async () => {
    try {
      if (isConnected || isDisconnecting) {
        await disconnect();
      } else if (!isConnecting) {
        await quickConnect();
      }
    } catch {
      // Error is already logged in the store and state is set to "error"
      // UI will reflect the error state automatically
    }
  };

  const handleQuickConnect = async () => {
    try {
      await quickConnect();
    } catch {
      // Error handled by store
    }
  };

  return (
    <div className="flex flex-col items-center justify-center h-full gap-6">
      {/* Connection Status Visual */}
      <motion.div
        className="relative"
        initial={{ scale: 0.9 }}
        animate={{ scale: 1 }}
        transition={{ type: "spring", stiffness: 300, damping: 20 }}
      >
        {/* Outer glow ring */}
        <motion.div
          className={clsx(
            "absolute inset-0 rounded-full blur-xl",
            isConnected ? "bg-emerald-500/30" : "bg-gray-500/20",
          )}
          animate={{
            scale: isConnected ? [1, 1.1, 1] : 1,
          }}
          transition={{
            duration: 2,
            repeat: isConnected ? Infinity : 0,
            ease: "easeInOut",
          }}
        />

        {/* Main button */}
        <motion.button
          onClick={handleMainButton}
          disabled={isConnecting || isDisconnecting}
          className={clsx(
            "relative w-36 h-36 rounded-full flex items-center justify-center",
            "transition-all duration-300 transform",
            "border-4",
            isConnected
              ? "bg-gradient-to-br from-emerald-500 to-emerald-600 border-emerald-400/50 shadow-2xl shadow-emerald-500/40"
              : "bg-gradient-to-br from-gray-700 to-gray-800 border-gray-600/50 hover:from-phantom-600 hover:to-phantom-700 hover:border-phantom-500/50",
            isConnecting && "animate-pulse cursor-wait",
          )}
          whileHover={{ scale: isConnecting ? 1 : 1.05 }}
          whileTap={{ scale: isConnecting ? 1 : 0.95 }}
        >
          {isConnected ? (
            <Shield className="w-16 h-16 text-white drop-shadow-lg" />
          ) : (
            <ShieldOff className="w-16 h-16 text-white/80" />
          )}
        </motion.button>
      </motion.div>

      {/* Status Text */}
      <div className="text-center">
        <motion.h1
          className={clsx(
            "text-2xl font-bold mb-1",
            isConnected ? "text-emerald-400" : "text-white",
          )}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
        >
          {isConnected
            ? "Protected"
            : isConnecting
              ? "Connecting..."
              : "Not Protected"}
        </motion.h1>
        <p className="text-sm text-white/60">
          {isConnected
            ? `Connected to ${currentServer?.city}, ${currentServer?.country}`
            : "Tap to connect"}
        </p>
      </div>

      {/* Quick Actions */}
      {!isConnected && !isConnecting && (
        <motion.div
          className="flex gap-3"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
        >
          <button
            onClick={handleQuickConnect}
            className="btn-primary px-6 py-3 flex items-center gap-2"
          >
            <Zap className="w-4 h-4" />
            Quick Connect
          </button>
        </motion.div>
      )}

      {/* Connection Info */}
      {isConnected && currentServer && (
        <motion.div
          className="glass rounded-2xl p-4 w-full max-w-xs"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
        >
          {/* Server info */}
          <div className="flex items-center gap-3 mb-4 pb-4 border-b border-white/10">
            <div className="w-10 h-10 rounded-full bg-white/10 flex items-center justify-center">
              <Globe className="w-5 h-5 text-phantom-400" />
            </div>
            <div>
              <p className="font-medium text-white">{currentServer.city}</p>
              <p className="text-xs text-white/60">
                {currentServer.country} • {currentServer.ipAddress}
              </p>
            </div>
          </div>

          {/* Stats grid */}
          <div className="grid grid-cols-2 gap-4">
            <div className="flex items-center gap-2">
              <ArrowDown className="w-4 h-4 text-emerald-400" />
              <div>
                <p className="text-xs text-white/60">Download</p>
                <p className="text-sm font-medium">
                  {formatBytes(connectionStats?.currentSpeed.download || 0)}/s
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <ArrowUp className="w-4 h-4 text-blue-400" />
              <div>
                <p className="text-xs text-white/60">Upload</p>
                <p className="text-sm font-medium">
                  {formatBytes(connectionStats?.currentSpeed.upload || 0)}/s
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <Clock className="w-4 h-4 text-purple-400" />
              <div>
                <p className="text-xs text-white/60">Duration</p>
                <p className="text-sm font-medium">
                  {formatDuration(connectionStats?.duration || 0)}
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              {settings.killSwitch ? (
                <Lock className="w-4 h-4 text-amber-400" />
              ) : (
                <Unlock className="w-4 h-4 text-gray-400" />
              )}
              <div>
                <p className="text-xs text-white/60">Kill Switch</p>
                <p className="text-sm font-medium">
                  {settings.killSwitch ? "Active" : "Inactive"}
                </p>
              </div>
            </div>
          </div>
        </motion.div>
      )}

      {/* Disconnect button when connected */}
      {isConnected && (
        <motion.button
          onClick={disconnect}
          className="btn-secondary px-6 py-2 text-red-400 border-red-500/30 hover:bg-red-500/10"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
        >
          Disconnect
        </motion.button>
      )}
    </div>
  );
}

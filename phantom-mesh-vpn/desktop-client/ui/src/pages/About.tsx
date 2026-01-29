import { motion } from "framer-motion";
import {
  Shield,
  Github,
  Globe,
  Heart,
  FileText,
  RefreshCw,
  ExternalLink,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";

export default function About() {
  const version = "1.0.0";

  const handleCheckUpdates = async () => {
    try {
      await invoke("check_updates");
    } catch (error) {
      console.error("Failed to check updates:", error);
    }
  };

  const handleOpenLink = (url: string) => {
    open(url);
  };

  return (
    <div className="flex flex-col items-center h-full overflow-auto pb-4">
      {/* Logo & Version */}
      <motion.div
        className="text-center mb-8 mt-4"
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
      >
        <div
          className="w-24 h-24 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-phantom-500 to-purple-600 
                        flex items-center justify-center shadow-2xl shadow-phantom-500/30"
        >
          <Shield className="w-12 h-12 text-white" />
        </div>
        <h1 className="text-2xl font-bold text-white">PhantomMesh VPN</h1>
        <p className="text-white/60 mt-1">Secure. Private. Unstoppable.</p>
        <p className="text-xs text-white/40 mt-2">Version {version}</p>
      </motion.div>

      {/* Info Cards */}
      <div className="w-full space-y-4 mb-6">
        <motion.div
          className="glass rounded-2xl p-4"
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.1 }}
        >
          <p className="text-sm text-white/80 leading-relaxed">
            PhantomMesh VPN provides enterprise-grade encryption with WireGuard,
            protecting your online privacy and securing your digital life across
            all devices.
          </p>
        </motion.div>

        {/* Stats */}
        <motion.div
          className="grid grid-cols-3 gap-3"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
        >
          <div className="glass rounded-xl p-3 text-center">
            <p className="text-2xl font-bold text-phantom-400">50+</p>
            <p className="text-xs text-white/50">Locations</p>
          </div>
          <div className="glass rounded-xl p-3 text-center">
            <p className="text-2xl font-bold text-purple-400">256</p>
            <p className="text-xs text-white/50">Bit Encryption</p>
          </div>
          <div className="glass rounded-xl p-3 text-center">
            <p className="text-2xl font-bold text-emerald-400">0</p>
            <p className="text-xs text-white/50">Logs Policy</p>
          </div>
        </motion.div>
      </div>

      {/* Actions */}
      <div className="w-full space-y-2 mb-6">
        <button
          onClick={handleCheckUpdates}
          className="w-full glass rounded-xl p-4 flex items-center gap-3 hover:bg-white/5 transition-colors"
        >
          <RefreshCw className="w-5 h-5 text-white/60" />
          <span className="flex-1 text-left text-white">Check for Updates</span>
        </button>

        <button
          onClick={() => handleOpenLink("https://phantommesh.io")}
          className="w-full glass rounded-xl p-4 flex items-center gap-3 hover:bg-white/5 transition-colors"
        >
          <Globe className="w-5 h-5 text-white/60" />
          <span className="flex-1 text-left text-white">Website</span>
          <ExternalLink className="w-4 h-4 text-white/30" />
        </button>

        <button
          onClick={() => handleOpenLink("https://github.com/phantommesh/vpn")}
          className="w-full glass rounded-xl p-4 flex items-center gap-3 hover:bg-white/5 transition-colors"
        >
          <Github className="w-5 h-5 text-white/60" />
          <span className="flex-1 text-left text-white">GitHub</span>
          <ExternalLink className="w-4 h-4 text-white/30" />
        </button>

        <button
          onClick={() => handleOpenLink("https://phantommesh.io/privacy")}
          className="w-full glass rounded-xl p-4 flex items-center gap-3 hover:bg-white/5 transition-colors"
        >
          <FileText className="w-5 h-5 text-white/60" />
          <span className="flex-1 text-left text-white">Privacy Policy</span>
          <ExternalLink className="w-4 h-4 text-white/30" />
        </button>
      </div>

      {/* Footer */}
      <motion.div
        className="text-center mt-auto"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.3 }}
      >
        <p className="text-xs text-white/30">
          Made with{" "}
          <Heart className="w-3 h-3 inline text-red-400 fill-red-400" /> by
          PhantomMesh Team
        </p>
        <p className="text-xs text-white/20 mt-1">
          © 2025 PhantomMesh. All rights reserved.
        </p>
      </motion.div>
    </div>
  );
}

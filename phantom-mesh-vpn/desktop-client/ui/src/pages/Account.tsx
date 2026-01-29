import { motion } from "framer-motion";
import {
  Mail,
  Calendar,
  CreditCard,
  Shield,
  ChevronRight,
  LogOut,
  ExternalLink,
} from "lucide-react";

export default function Account() {
  // Mock account data - would come from backend
  const account = {
    email: "user@example.com",
    plan: "Premium",
    expiresAt: new Date("2025-12-31"),
    devices: 3,
    maxDevices: 10,
  };

  const daysRemaining = Math.ceil(
    (account.expiresAt.getTime() - Date.now()) / (1000 * 60 * 60 * 24),
  );

  return (
    <div className="space-y-6 overflow-auto h-full pb-4">
      {/* Profile Card */}
      <motion.div
        className="glass rounded-2xl p-6"
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
      >
        <div className="flex items-center gap-4 mb-6">
          <div
            className="w-16 h-16 rounded-full bg-gradient-to-br from-phantom-500 to-purple-600 
                          flex items-center justify-center text-2xl font-bold"
          >
            {account.email[0].toUpperCase()}
          </div>
          <div>
            <p className="text-lg font-semibold text-white">{account.email}</p>
            <div className="flex items-center gap-2 mt-1">
              <span className="text-xs bg-phantom-500/20 text-phantom-400 px-2 py-0.5 rounded-full">
                {account.plan}
              </span>
            </div>
          </div>
        </div>

        {/* Plan details */}
        <div className="grid grid-cols-2 gap-4">
          <div className="bg-white/5 rounded-xl p-3">
            <div className="flex items-center gap-2 text-white/60 mb-1">
              <Calendar className="w-4 h-4" />
              <span className="text-xs">Expires</span>
            </div>
            <p className="text-sm font-medium text-white">
              {account.expiresAt.toLocaleDateString()}
            </p>
            <p className="text-xs text-emerald-400 mt-0.5">
              {daysRemaining} days remaining
            </p>
          </div>
          <div className="bg-white/5 rounded-xl p-3">
            <div className="flex items-center gap-2 text-white/60 mb-1">
              <Shield className="w-4 h-4" />
              <span className="text-xs">Devices</span>
            </div>
            <p className="text-sm font-medium text-white">
              {account.devices} / {account.maxDevices}
            </p>
            <p className="text-xs text-white/40 mt-0.5">devices active</p>
          </div>
        </div>
      </motion.div>

      {/* Actions */}
      <section>
        <h2 className="text-sm font-semibold text-white/40 uppercase tracking-wider mb-3 px-4">
          Account
        </h2>
        <div className="glass rounded-2xl overflow-hidden divide-y divide-white/5">
          <button className="w-full flex items-center gap-4 p-4 hover:bg-white/5 transition-colors">
            <div className="w-10 h-10 rounded-lg flex items-center justify-center bg-white/5">
              <CreditCard className="w-5 h-5 text-white/60" />
            </div>
            <div className="flex-1 text-left">
              <p className="font-medium text-white">Manage Subscription</p>
              <p className="text-xs text-white/50 mt-0.5">
                Update payment & billing
              </p>
            </div>
            <ExternalLink className="w-4 h-4 text-white/30" />
          </button>

          <button className="w-full flex items-center gap-4 p-4 hover:bg-white/5 transition-colors">
            <div className="w-10 h-10 rounded-lg flex items-center justify-center bg-white/5">
              <Shield className="w-5 h-5 text-white/60" />
            </div>
            <div className="flex-1 text-left">
              <p className="font-medium text-white">Active Sessions</p>
              <p className="text-xs text-white/50 mt-0.5">
                View connected devices
              </p>
            </div>
            <ChevronRight className="w-5 h-5 text-white/30" />
          </button>

          <button className="w-full flex items-center gap-4 p-4 hover:bg-white/5 transition-colors">
            <div className="w-10 h-10 rounded-lg flex items-center justify-center bg-white/5">
              <Mail className="w-5 h-5 text-white/60" />
            </div>
            <div className="flex-1 text-left">
              <p className="font-medium text-white">Change Email</p>
              <p className="text-xs text-white/50 mt-0.5">{account.email}</p>
            </div>
            <ChevronRight className="w-5 h-5 text-white/30" />
          </button>
        </div>
      </section>

      {/* Logout */}
      <section>
        <button
          className="w-full glass rounded-2xl p-4 flex items-center gap-4 
                          hover:bg-red-500/10 transition-colors group"
        >
          <div className="w-10 h-10 rounded-lg flex items-center justify-center bg-red-500/10">
            <LogOut className="w-5 h-5 text-red-400" />
          </div>
          <div className="flex-1 text-left">
            <p className="font-medium text-red-400">Sign Out</p>
            <p className="text-xs text-white/50 mt-0.5">
              Disconnect from this device
            </p>
          </div>
        </button>
      </section>
    </div>
  );
}

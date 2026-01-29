import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import {
  Search,
  Star,
  StarOff,
  Signal,
  Globe,
  Shield,
  Play,
  Download,
} from "lucide-react";
import clsx from "clsx";
import { useVpnStore, ServerInfo } from "../stores/vpnStore";

const countryFlags: Record<string, string> = {
  US: "🇺🇸",
  GB: "🇬🇧",
  DE: "🇩🇪",
  NL: "🇳🇱",
  JP: "🇯🇵",
  SG: "🇸🇬",
  AU: "🇦🇺",
  CA: "🇨🇦",
  FR: "🇫🇷",
  CH: "🇨🇭",
};

const featureIcons: Record<string, typeof Shield> = {
  p2p: Download,
  streaming: Play,
  secure_core: Shield,
  tor: Globe,
};

export default function Servers() {
  const {
    servers,
    favoriteServers,
    recentServers,
    connectionState,
    currentServer,
    fetchServers,
    connect,
    toggleFavorite,
  } = useVpnStore();

  const [searchQuery, setSearchQuery] = useState("");
  const [filter, setFilter] = useState<"all" | "favorites" | "recent">("all");

  useEffect(() => {
    fetchServers();
  }, [fetchServers]);

  const filteredServers = servers.filter((server) => {
    // Search filter
    const matchesSearch =
      server.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      server.country.toLowerCase().includes(searchQuery.toLowerCase()) ||
      server.city.toLowerCase().includes(searchQuery.toLowerCase());

    // Category filter
    if (filter === "favorites") {
      return matchesSearch && favoriteServers.includes(server.id);
    }
    if (filter === "recent") {
      return matchesSearch && recentServers.includes(server.id);
    }

    return matchesSearch;
  });

  // Group servers by country
  const groupedServers = filteredServers.reduce(
    (acc, server) => {
      if (!acc[server.country]) {
        acc[server.country] = [];
      }
      acc[server.country].push(server);
      return acc;
    },
    {} as Record<string, ServerInfo[]>,
  );

  const isConnecting = connectionState === "connecting";

  const handleConnect = async (serverId: string) => {
    try {
      await connect(serverId);
    } catch {
      // Error logged in store
    }
  };

  const getLoadColor = (load: number) => {
    if (load < 30) return "text-emerald-400";
    if (load < 70) return "text-amber-400";
    return "text-red-400";
  };

  const getLoadBg = (load: number) => {
    if (load < 30) return "bg-emerald-500";
    if (load < 70) return "bg-amber-500";
    return "bg-red-500";
  };

  return (
    <div className="flex flex-col h-full">
      {/* Search Bar */}
      <div className="relative mb-4">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-white/40" />
        <input
          type="text"
          placeholder="Search servers..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full pl-10 pr-4 py-2.5 bg-white/5 border border-white/10 rounded-xl 
                     text-white placeholder-white/40 focus:outline-none focus:border-phantom-500/50
                     transition-colors"
        />
      </div>

      {/* Filter Tabs */}
      <div className="flex gap-2 mb-4">
        {(["all", "favorites", "recent"] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setFilter(tab)}
            className={clsx(
              "px-4 py-1.5 rounded-lg text-sm font-medium transition-all capitalize",
              filter === tab
                ? "bg-phantom-500/20 text-phantom-400 border border-phantom-500/30"
                : "text-white/60 hover:text-white hover:bg-white/5",
            )}
          >
            {tab === "favorites" && <Star className="w-3 h-3 inline mr-1" />}
            {tab}
          </button>
        ))}
      </div>

      {/* Server List */}
      <div className="flex-1 overflow-auto space-y-4 -mr-2 pr-2">
        {Object.entries(groupedServers).map(([country, countryServers]) => (
          <motion.div
            key={country}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
          >
            {/* Country Header */}
            <div className="flex items-center gap-2 mb-2">
              <span className="text-lg">
                {countryFlags[countryServers[0].countryCode] || "🌍"}
              </span>
              <span className="text-sm font-medium text-white/80">
                {country}
              </span>
              <span className="text-xs text-white/40">
                ({countryServers.length})
              </span>
            </div>

            {/* Server Cards */}
            <div className="space-y-2">
              {countryServers.map((server) => {
                const isFavorite = favoriteServers.includes(server.id);
                const isCurrentServer = currentServer?.id === server.id;

                return (
                  <motion.button
                    key={server.id}
                    onClick={() => handleConnect(server.id)}
                    disabled={isConnecting}
                    className={clsx(
                      "w-full glass rounded-xl p-3 flex items-center gap-3",
                      "transition-all duration-200 text-left",
                      "hover:bg-white/10 hover:border-white/20",
                      isCurrentServer &&
                        "border-emerald-500/50 bg-emerald-500/10",
                    )}
                    whileHover={{ scale: 1.01 }}
                    whileTap={{ scale: 0.99 }}
                  >
                    {/* Server icon */}
                    <div
                      className={clsx(
                        "w-10 h-10 rounded-lg flex items-center justify-center",
                        isCurrentServer ? "bg-emerald-500/20" : "bg-white/5",
                      )}
                    >
                      <Signal
                        className={clsx(
                          "w-5 h-5",
                          isCurrentServer
                            ? "text-emerald-400"
                            : "text-white/60",
                        )}
                      />
                    </div>

                    {/* Server info */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <p className="font-medium text-white truncate">
                          {server.city}
                        </p>
                        {isCurrentServer && (
                          <span className="text-xs bg-emerald-500/20 text-emerald-400 px-2 py-0.5 rounded-full">
                            Connected
                          </span>
                        )}
                      </div>
                      <div className="flex items-center gap-3 mt-0.5">
                        <span className="text-xs text-white/50">
                          {server.name}
                        </span>
                        {/* Features */}
                        <div className="flex gap-1">
                          {server.features.map((feature) => {
                            const Icon = featureIcons[feature] || Globe;
                            return (
                              <span key={feature} title={feature}>
                                <Icon className="w-3 h-3 text-white/30" />
                              </span>
                            );
                          })}
                        </div>
                      </div>
                    </div>

                    {/* Load indicator */}
                    <div className="flex items-center gap-2">
                      <div className="text-right">
                        <p
                          className={clsx(
                            "text-sm font-medium",
                            getLoadColor(server.load),
                          )}
                        >
                          {server.load}%
                        </p>
                        <div className="w-12 h-1 bg-white/10 rounded-full overflow-hidden">
                          <div
                            className={clsx(
                              "h-full rounded-full",
                              getLoadBg(server.load),
                            )}
                            style={{ width: `${server.load}%` }}
                          />
                        </div>
                      </div>

                      {/* Favorite button */}
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          toggleFavorite(server.id);
                        }}
                        className="p-1.5 hover:bg-white/10 rounded-lg transition-colors"
                      >
                        {isFavorite ? (
                          <Star className="w-4 h-4 text-amber-400 fill-amber-400" />
                        ) : (
                          <StarOff className="w-4 h-4 text-white/30 hover:text-white/60" />
                        )}
                      </button>
                    </div>
                  </motion.button>
                );
              })}
            </div>
          </motion.div>
        ))}

        {filteredServers.length === 0 && (
          <div className="flex flex-col items-center justify-center h-40 text-white/40">
            <Globe className="w-12 h-12 mb-2 opacity-50" />
            <p>No servers found</p>
          </div>
        )}
      </div>
    </div>
  );
}

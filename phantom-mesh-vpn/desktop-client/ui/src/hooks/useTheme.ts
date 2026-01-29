import { useCallback, useEffect, useState } from "react";

type Theme = "dark" | "light" | "system";

export function useTheme() {
  const [theme, setTheme] = useState<Theme>("dark");

  // For now, always use dark theme
  useEffect(() => {
    document.documentElement.classList.add("dark");
  }, []);

  const toggleTheme = useCallback(() => {
    // No-op for now, always dark
  }, []);

  return { theme, setTheme, toggleTheme };
}

import { useState, useEffect, useCallback } from 'react';

const DEFAULT_SYMBOLS = ['SPX', 'XSP', 'NANOS'];
const STORAGE_KEY = 'symbol_watchlist';
const MAX_SYMBOLS = 50; // Maximum custom symbols

/**
 * Validate symbol format
 * Rules: 1-5 uppercase alphanumeric characters
 */
function validateSymbol(symbol: string): { valid: boolean; error?: string } {
  const trimmed = symbol.trim().toUpperCase();

  if (!trimmed) {
    return { valid: false, error: 'Symbol cannot be empty' };
  }

  if (trimmed.length > 5) {
    return { valid: false, error: 'Symbol must be 5 characters or less' };
  }

  // Allow uppercase letters, numbers, and dots (for symbols like BRK.B)
  if (!/^[A-Z0-9.]+$/.test(trimmed)) {
    return { valid: false, error: 'Symbol must contain only letters, numbers, and dots' };
  }

  return { valid: true };
}

/**
 * Load custom symbols from localStorage
 */
function loadCustomSymbols(): string[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (!stored) return [];

    const parsed = JSON.parse(stored);
    if (Array.isArray(parsed)) {
      // Validate and filter
      return parsed
        .filter((s) => typeof s === 'string')
        .map((s) => s.trim().toUpperCase())
        .filter((s) => s.length > 0 && s.length <= 5);
    }
    return [];
  } catch {
    return [];
  }
}

/**
 * Save custom symbols to localStorage
 */
function saveCustomSymbols(symbols: string[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(symbols));
  } catch (error) {
    console.warn('Failed to save symbols to localStorage:', error);
  }
}

interface UseSymbolWatchlistReturn {
  watchlist: string[]; // All symbols (defaults + custom)
  customSymbols: string[]; // Only custom symbols
  addSymbol: (symbol: string) => { success: boolean; error?: string };
  removeSymbol: (symbol: string) => void;
  isDefault: (symbol: string) => boolean;
}

export function useSymbolWatchlist(): UseSymbolWatchlistReturn {
  const [customSymbols, setCustomSymbols] = useState<string[]>(() => loadCustomSymbols());

  // Merge defaults with custom symbols
  const watchlist = useCallback(() => {
    const all = [...DEFAULT_SYMBOLS];
    // Add custom symbols that aren't already in defaults
    customSymbols.forEach((symbol) => {
      if (!DEFAULT_SYMBOLS.includes(symbol)) {
        all.push(symbol);
      }
    });
    return all;
  }, [customSymbols])();

  // Save to localStorage when custom symbols change
  useEffect(() => {
    saveCustomSymbols(customSymbols);
  }, [customSymbols]);

  const addSymbol = useCallback(
    (symbol: string): { success: boolean; error?: string } => {
      const validation = validateSymbol(symbol);
      if (!validation.valid) {
        return { success: false, error: validation.error };
      }

      const normalized = symbol.trim().toUpperCase();

      // Check if already in watchlist (defaults or custom)
      if (watchlist.includes(normalized)) {
        return { success: false, error: 'Symbol already in watchlist' };
      }

      // Check if max symbols reached
      if (customSymbols.length >= MAX_SYMBOLS) {
        return { success: false, error: `Maximum ${MAX_SYMBOLS} custom symbols allowed` };
      }

      setCustomSymbols((prev) => [...prev, normalized]);
      return { success: true };
    },
    [watchlist, customSymbols.length]
  );

  const removeSymbol = useCallback((symbol: string) => {
    const normalized = symbol.trim().toUpperCase();

    // Cannot remove default symbols
    if (DEFAULT_SYMBOLS.includes(normalized)) {
      return;
    }

    setCustomSymbols((prev) => prev.filter((s) => s !== normalized));
  }, []);

  const isDefault = useCallback((symbol: string): boolean => {
    return DEFAULT_SYMBOLS.includes(symbol.trim().toUpperCase());
  }, []);

  return {
    watchlist,
    customSymbols,
    addSymbol,
    removeSymbol,
    isDefault
  };
}

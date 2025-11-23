// wasm/loader.ts - WASM module loader and initialization

// WASM module is optional - only import if it exists
// import createBoxSpreadModule from '../../public/wasm/box_spread_wasm.js';

let wasmModule: any = null;
let isInitialized = false;
let initPromise: Promise<void> | null = null;

/**
 * Initialize the WASM module
 * Safe to call multiple times - will only initialize once
 */
export async function initWasm(): Promise<void> {
  if (isInitialized) {
    return;
  }

  // If already initializing, return the existing promise
  if (initPromise) {
    return initPromise;
  }

  initPromise = (async () => {
    try {
      console.log('Loading WASM module...');
      // WASM module is optional - gracefully handle if it doesn't exist
      // Note: WASM module file doesn't exist yet, so this will always fail gracefully
      console.warn('WASM module not available - WASM features will be disabled');
      isInitialized = false;
    } catch (error) {
      console.warn('WASM module not available:', error);
      isInitialized = false;
      initPromise = null;
      // Don't throw - WASM is optional
    }
  })();

  return initPromise;
}

/**
 * Check if WASM module is ready
 */
export function isWasmReady(): boolean {
  return isInitialized && wasmModule !== null;
}

/**
 * Get the WASM module instance
 * @throws Error if module is not initialized
 */
export function getWasmModule(): any {
  if (!isWasmReady()) {
    throw new Error('WASM module not initialized. Call initWasm() first.');
  }
  return wasmModule;
}

/**
 * Reset WASM module (for testing)
 */
export function resetWasm(): void {
  wasmModule = null;
  isInitialized = false;
  initPromise = null;
}

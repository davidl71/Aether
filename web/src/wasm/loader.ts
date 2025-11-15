// wasm/loader.ts - WASM module loader and initialization

import createBoxSpreadModule from '../../public/wasm/box_spread_wasm.js';

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
      wasmModule = await createBoxSpreadModule();
      isInitialized = true;
      console.log('✅ WASM module loaded successfully');
    } catch (error) {
      console.error('❌ Failed to load WASM module:', error);
      isInitialized = false;
      initPromise = null;
      throw error;
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

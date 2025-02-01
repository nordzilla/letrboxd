export interface WasmWorker {
  ready: Promise<void>;
  resolveReadyPromise: () => void;
  hasPendingRequests: boolean;
}
/**
 * UI-Zustand des Downloads-Managers (T-109).
 *
 * Der Store besitzt nur den Fenster-Zustand (offen/geschlossen) nach dem
 * Vorbild von `bookmarkStore.isManagerOpen`. Die Download-Daten selbst
 * kommen live aus dem Backend via `ui/src/api/downloads.ts`.
 */

interface DownloadUiState {
  isManagerOpen: boolean;
}

function createDefaultState(): DownloadUiState {
  return { isManagerOpen: false };
}

class DownloadStore {
  private state: DownloadUiState = createDefaultState();
  private listeners = new Set<(state: DownloadUiState) => void>();

  private notify(): void {
    const current = this.getState();
    this.listeners.forEach((fn) => fn(current));
  }

  public getState(): DownloadUiState {
    return { ...this.state };
  }

  public subscribe(callback: (state: DownloadUiState) => void): () => void {
    this.listeners.add(callback);
    callback(this.getState());
    return () => {
      this.listeners.delete(callback);
    };
  }

  public get isManagerOpen(): boolean {
    return this.state.isManagerOpen;
  }

  public openManager(): void {
    if (!this.state.isManagerOpen) {
      this.state.isManagerOpen = true;
      this.notify();
    }
  }

  public closeManager(): void {
    if (this.state.isManagerOpen) {
      this.state.isManagerOpen = false;
      this.notify();
    }
  }

  /** Nur fuer Tests. */
  public resetForTests(): void {
    this.state = createDefaultState();
    this.notify();
  }
}

export const downloadStore = new DownloadStore();

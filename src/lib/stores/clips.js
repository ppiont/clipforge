import { writable } from 'svelte/store';

/**
 * Media library clips store
 * Stores imported video files with metadata
 */
export const clipsStore = writable([
  // Clip structure:
  // {
  //   id: string,
  //   filename: string,
  //   path: string,
  //   duration: number,
  //   resolution: string (e.g., "1920x1080")
  // }
]);

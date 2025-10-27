import { writable } from 'svelte/store';

/**
 * Timeline clips store
 * Stores clips placed on the timeline with position and trim data
 */
export const timelineStore = writable({
  clips: [
    // Timeline clip structure:
    // {
    //   id: string (unique timeline clip ID),
    //   clipId: string (reference to clips store),
    //   track: number (0 = main, 1 = overlay),
    //   startTime: number (seconds, position on timeline),
    //   trimStart: number (seconds, trim in point),
    //   trimEnd: number (seconds, trim out point),
    //   duration: number (seconds, trimEnd - trimStart)
    // }
  ],
  playhead: 0, // Current playhead position in seconds
  duration: 0  // Total timeline duration in seconds
});

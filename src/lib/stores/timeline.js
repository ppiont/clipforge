import { writable } from 'svelte/store';

/**
 * @typedef {Object} TimelineClip
 * @property {string} id - Unique timeline clip ID
 * @property {string} clipId - Reference to clips store
 * @property {number} track - 0 = main, 1 = overlay
 * @property {number} startTime - Position on timeline in seconds
 * @property {number} trimStart - Trim in point in seconds
 * @property {number} trimEnd - Trim out point in seconds
 * @property {number} duration - Duration in seconds
 */

/**
 * @typedef {Object} TimelineState
 * @property {TimelineClip[]} clips - Clips on timeline
 * @property {number} playhead - Current playhead position in seconds
 * @property {number} duration - Total timeline duration in seconds
 */

/** @type {import('svelte/store').Writable<TimelineState>} */
export const timelineStore = writable({
  clips: [],
  playhead: 0,
  duration: 0
});

// Export types for use in other modules
export {};

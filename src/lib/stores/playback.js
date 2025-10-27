import { writable } from 'svelte/store';

/**
 * @typedef {Object} PlaybackState
 * @property {boolean} isPlaying - Whether video is currently playing
 * @property {number} currentTime - Current playback position in seconds
 * @property {string | null} selectedClipId - Currently selected clip ID from media library
 * @property {string | null} selectedTimelineClipId - Currently selected clip on timeline
 */

/** @type {import('svelte/store').Writable<PlaybackState>} */
export const playbackStore = writable({
  isPlaying: false,
  currentTime: 0,
  selectedClipId: null,
  selectedTimelineClipId: null
});

import { writable } from 'svelte/store';

/**
 * Playback state store
 * Tracks video playback status and selected clips
 */
export const playbackStore = writable({
  isPlaying: false,           // Whether video is currently playing
  currentTime: 0,             // Current playback position in seconds
  selectedClipId: null,       // Currently selected clip ID from media library
  selectedTimelineClipId: null // Currently selected clip on timeline
});

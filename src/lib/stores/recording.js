import { writable } from 'svelte/store';

/**
 * @typedef {Object} RecordingState
 * @property {boolean} isRecording - Whether currently recording
 * @property {'screen' | 'webcam' | 'both' | null} source - Recording source
 * @property {number | null} startTime - Recording start timestamp
 * @property {MediaStream | null} stream - MediaStream object
 * @property {number} recordedDuration - Duration of current recording in seconds
 */

/** @type {import('svelte/store').Writable<RecordingState>} */
export const recordingStore = writable({
  isRecording: false,
  source: null,
  startTime: null,
  stream: null,
  recordedDuration: 0
});

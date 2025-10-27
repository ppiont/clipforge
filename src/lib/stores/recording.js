import { writable } from 'svelte/store';

/**
 * Recording state store
 * Tracks recording session status and parameters
 * (Stub for future recording features)
 */
export const recordingStore = writable({
  isRecording: false,     // Whether currently recording
  source: null,           // 'screen' | 'webcam' | 'both' | null
  startTime: null,        // Recording start timestamp
  stream: null,           // MediaStream object
  recordedDuration: 0     // Duration of current recording in seconds
});

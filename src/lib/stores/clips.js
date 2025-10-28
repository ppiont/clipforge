import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

/**
 * @typedef {Object} Clip
 * @property {string} id - Unique clip ID
 * @property {string} filename - Original filename
 * @property {string} path - File path
 * @property {number} duration - Duration in seconds
 * @property {string} resolution - Resolution string (e.g., "1920x1080")
 * @property {string} [codec] - Optional video codec
 * @property {string} [thumbnail] - Optional thumbnail data URL (base64-encoded image)
 * @property {string} [filmstrip] - Optional filmstrip file path
 * @property {number} [filmstripFrameCount] - Number of frames in filmstrip (default: 20)
 */

/** @type {import('svelte/store').Writable<Clip[]>} */
export const clipsStore = writable(
  /** @type {Clip[]} */ []
);

/**
 * Generate filmstrip for a clip (if not already generated)
 * @param {string} clipId - The clip ID to generate filmstrip for
 * @param {number} frameCount - Number of frames to generate (default: 20)
 * @returns {Promise<void>}
 */
export async function generateFilmstripForClip(clipId, frameCount = 20) {
  const clips = get(clipsStore);
  const clip = clips.find(c => c.id === clipId);

  if (!clip) {
    console.warn(`Clip ${clipId} not found`);
    return;
  }

  // Skip if filmstrip already exists
  if (clip.filmstrip) {
    console.log(`Filmstrip already exists for clip ${clipId}`);
    return;
  }

  try {
    console.log(`Generating filmstrip for clip ${clipId}...`);
    const filmstripPath = await invoke('generate_filmstrip', {
      videoPath: clip.path,
      clipId: clip.id,
      frameCount
    });

    // Update clip with filmstrip path
    clipsStore.update(clips =>
      clips.map(c =>
        c.id === clipId
          ? { ...c, filmstrip: filmstripPath, filmstripFrameCount: frameCount }
          : c
      )
    );

    console.log(`Filmstrip generated successfully for clip ${clipId}`);
  } catch (err) {
    console.error(`Filmstrip generation failed for clip ${clipId}:`, err);
  }
}

// Export types for use in other modules
export {};

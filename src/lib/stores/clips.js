import { writable } from 'svelte/store';

/**
 * @typedef {Object} Clip
 * @property {string} id - Unique clip ID
 * @property {string} filename - Original filename
 * @property {string} path - File path
 * @property {number} duration - Duration in seconds
 * @property {string} resolution - Resolution string (e.g., "1920x1080")
 */

/** @type {import('svelte/store').Writable<Clip[]>} */
export const clipsStore = writable(
  /** @type {Clip[]} */ []
);

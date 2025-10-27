<script>
  import { onMount } from 'svelte';
  import { playbackStore } from '../stores/playback.js';

  /**
   * Controls Component
   * Playback controls and timeline editing buttons
   */

  /** @type {HTMLVideoElement | null} */
  export let videoElement = null;

  function togglePlayPause() {
    if (!videoElement) return;

    if ($playbackStore.isPlaying) {
      videoElement.pause();
      playbackStore.update(state => ({ ...state, isPlaying: false }));
    } else {
      videoElement.play();
      playbackStore.update(state => ({ ...state, isPlaying: true }));
    }
  }

  function handleStop() {
    if (!videoElement) return;
    videoElement.pause();
    videoElement.currentTime = 0;
    playbackStore.update(state => ({
      ...state,
      isPlaying: false,
      currentTime: 0
    }));
  }

  /** @param {KeyboardEvent} e */
  function handleKeyPress(e) {
    if (e.code === 'Space') {
      e.preventDefault();
      togglePlayPause();
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeyPress);
    return () => {
      window.removeEventListener('keydown', handleKeyPress);
    };
  });
</script>

<div class="controls">
  <div class="playback-controls">
    <button
      class="play-button"
      on:click={togglePlayPause}
      title="Play/Pause (Space)"
    >
      {$playbackStore.isPlaying ? '⏸ Pause' : '▶ Play'}
    </button>

    <button on:click={handleStop} title="Stop">
      ⏹ Stop
    </button>
  </div>

  <div class="editing-controls">
    <button title="Split clip (Coming soon)" disabled>
      ✂ Split
    </button>
    <button title="Delete clip (Coming soon)" disabled>
      🗑 Delete
    </button>
  </div>

  <div class="spacer"></div>

  <div class="info">
    <span class="state-label">
      {$playbackStore.isPlaying ? '▶ Playing' : '⏸ Paused'}
    </span>
  </div>
</div>

<style>
  .controls {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 50px;
    padding: 0 15px;
    background: #f5f5f5;
    border-top: 1px solid #ddd;
  }

  .playback-controls,
  .editing-controls {
    display: flex;
    gap: 5px;
  }

  button {
    padding: 8px 12px;
    background: #fff;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.2s;
  }

  button:hover:not(:disabled) {
    background: #f0f0f0;
    border-color: #999;
  }

  button:active:not(:disabled) {
    background: #e0e0e0;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .play-button {
    font-weight: 600;
    color: #1976d2;
    border-color: #1976d2;
  }

  .play-button:hover:not(:disabled) {
    background: #e3f2fd;
  }

  .spacer {
    flex: 1;
  }

  .info {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .state-label {
    font-size: 12px;
    color: #666;
  }
</style>

<script>
  import { onMount } from 'svelte';
  import { playbackStore } from '../stores/playback.js';
  import { Button } from "$lib/components/ui/button";

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

<footer class="flex items-center gap-3 h-[50px] px-4 bg-background border-t">
  <div class="flex gap-2">
    <Button
      variant="default"
      size="sm"
      on:click={togglePlayPause}
      title="Play/Pause (Space)"
    >
      {$playbackStore.isPlaying ? '⏸ Pause' : '▶ Play'}
    </Button>

    <Button
      variant="outline"
      size="sm"
      on:click={handleStop}
      title="Stop"
    >
      ⏹ Stop
    </Button>
  </div>

  <div class="flex gap-2">
    <Button variant="outline" size="sm" disabled title="Split clip (Coming soon)">
      ✂ Split
    </Button>
    <Button variant="outline" size="sm" disabled title="Delete clip (Coming soon)">
      🗑 Delete
    </Button>
  </div>

  <div class="flex-1"></div>

  <div class="text-xs text-muted-foreground">
    {$playbackStore.isPlaying ? '▶ Playing' : '⏸ Paused'}
  </div>
</footer>

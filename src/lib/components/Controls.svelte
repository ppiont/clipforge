<script>
  import { onMount } from 'svelte';
  import { playbackStore } from '../stores/playback.js';
  import { Button } from "$lib/components/ui/button";
  import { Play, Pause, Square, Scissors, Trash2 } from "@lucide/svelte";

  /**
   * Controls Component
   * Playback controls and timeline editing buttons
   */

  let {
    videoElement = $bindable(null)
  } = $props();

  function togglePlayPause() {
    if (!videoElement) {
      console.error("No video element found");
      return;
    }

    console.log("Toggle play/pause, current state:", $playbackStore.isPlaying);

    if ($playbackStore.isPlaying) {
      videoElement.pause();
      playbackStore.update(state => ({ ...state, isPlaying: false }));
    } else {
      videoElement.play().catch((/** @type {any} */ err) => {
        console.error("Error playing video:", err);
      });
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
      class=""
      variant="default"
      size="sm"
      disabled={false}
      onclick={togglePlayPause}
      title="Play/Pause (Space)"
    >
      {#if $playbackStore.isPlaying}
        <Pause class="w-4 h-4" />
        Pause
      {:else}
        <Play class="w-4 h-4" />
        Play
      {/if}
    </Button>

    <Button
      class=""
      variant="outline"
      size="sm"
      disabled={false}
      onclick={handleStop}
      title="Stop"
    >
      <Square class="w-4 h-4" />
      Stop
    </Button>
  </div>

  <div class="flex gap-2">
    <Button
      class=""
      variant="outline"
      size="sm"
      disabled={true}
      title="Split clip (Coming soon)"
    >
      <Scissors class="w-4 h-4" />
      Split
    </Button>
    <Button
      class=""
      variant="outline"
      size="sm"
      disabled={true}
      title="Delete clip (Coming soon)"
    >
      <Trash2 class="w-4 h-4" />
      Delete
    </Button>
  </div>

  <div class="flex-1"></div>

  <div class="flex items-center gap-1 text-xs text-muted-foreground">
    {#if $playbackStore.isPlaying}
      <Play class="w-3 h-3" />
      Playing
    {:else}
      <Pause class="w-3 h-3" />
      Paused
    {/if}
  </div>
</footer>

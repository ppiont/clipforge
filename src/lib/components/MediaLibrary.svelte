<script>
  import { clipsStore } from "../stores/clips.js";
  import { playbackStore } from "../stores/playback.js";
  import { Card } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { ScrollArea } from "$lib/components/ui/scroll-area";

  /**
   * MediaLibrary Component
   * Shows list of imported video clips
   * Allows selection and drag-to-timeline
   */

  /** @param {string} clipId */
  function selectClip(clipId) {
    playbackStore.update((state) => ({
      ...state,
      selectedClipId: clipId,
      selectedTimelineClipId: null,
    }));
  }

  /** @param {number} seconds */
  function formatTime(seconds) {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  /** @param {DragEvent} e
   *  @param {{id: string, duration: number, path: string}} clip */
  function handleDragStart(e, clip) {
    if (!e.dataTransfer) return;
    e.dataTransfer.effectAllowed = "copy";
    e.dataTransfer.setData(
      "application/json",
      JSON.stringify({
        clipId: clip.id,
        duration: clip.duration,
        path: clip.path,
      }),
    );
  }
</script>

<div class="flex flex-col h-full border-l">
  <div class="px-4 py-3 border-b bg-muted">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-semibold">Media Library</h3>
      <Badge variant="secondary" class="" href={undefined}
        >{$clipsStore.length}</Badge
      >
    </div>
  </div>

  <ScrollArea class="flex-1">
    <div class="p-3 space-y-2">
      {#each $clipsStore as clip (clip.id)}
        <Card
          class={`p-3 cursor-pointer transition-all hover:shadow-md ${
            $playbackStore.selectedClipId === clip.id
              ? "ring-2 ring-primary bg-primary/5"
              : "hover:bg-muted"
          }`}
          draggable="true"
          on:click={() => selectClip(clip.id)}
          on:keydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              selectClip(clip.id);
            }
          }}
          on:dragstart={(e) => handleDragStart(e, clip)}
          role="button"
          tabindex="0"
        >
          <div class="flex gap-3">
            <div
              class="shrink-0 w-10 h-10 bg-muted rounded flex items-center justify-center text-lg"
            >
              🎬
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate">{clip.filename}</p>
              <p class="text-xs text-muted-foreground">
                {formatTime(clip.duration)} • {clip.resolution}
              </p>
            </div>
          </div>
        </Card>
      {/each}

      {#if $clipsStore.length === 0}
        <div
          class="flex flex-col items-center justify-center py-12 text-center text-muted-foreground"
        >
          <p class="text-sm">No clips imported yet</p>
          <p class="text-xs mt-1">Click Import to add videos</p>
        </div>
      {/if}
    </div>
  </ScrollArea>
</div>

<script>
  import { clipsStore } from "../stores/clips.js";
  import { playbackStore } from "../stores/playback.js";
  import { Card } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Button } from "$lib/components/ui/button";
  import { invoke } from "@tauri-apps/api/core";
  import { Video, X, Plus } from "@lucide/svelte";
  import { timelineStore } from "../stores/timeline.js";

  /**
   * MediaLibrary Component
   * Shows list of imported video clips
   * Allows selection and drag-to-timeline
   */

  let isDraggingOver = $state(false);

  /** @param {string} clipId */
  function selectClip(clipId) {
    /** @type {any} */
    const store = playbackStore;
    store.update((/** @type {any} */ state) => ({
      ...state,
      // Toggle selection: if already selected, deselect it
      selectedClipId: state.selectedClipId === clipId ? null : clipId,
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
   *  @param {{id: string, filename: string, duration: number, path: string}} clip */
  function handleDragStart(e, clip) {
    if (!e.dataTransfer) return;
    console.log("Drag started for clip:", clip.id, clip.filename);
    e.dataTransfer.effectAllowed = "copy";

    // Set both formats to try to bypass Tauri interception
    e.dataTransfer.setData("text/plain", clip.id);
    e.dataTransfer.setData(
      "application/json",
      JSON.stringify({
        clipId: clip.id,
        duration: clip.duration,
        path: clip.path,
      }),
    );

    // Mark as internal drag
    e.dataTransfer.setData("text/x-clipforge-clip", "true");
  }

  /** @param {string} clipId */
  function removeClip(clipId) {
    // @ts-ignore - Store is properly typed, TypeScript inference issue with writable()
    clipsStore.update((clips) => clips.filter((c) => c.id !== clipId));

    // If this was the selected clip, clear selection
    if ($playbackStore.selectedClipId === clipId) {
      // @ts-ignore - Store is properly typed, TypeScript inference issue with writable()
      playbackStore.update((state) => ({
        ...state,
        selectedClipId: null,
      }));
    }
  }

  /**
   * Handle files dropped via Tauri's drag-drop event
   * @param {string[]} paths - Array of file paths
   */
  async function handleFileDrop(paths) {
    isDraggingOver = false;

    for (const filePath of paths) {
      // Check if file is a video by extension
      const isVideo = /\.(mp4|mov|webm|mkv|avi)$/i.test(filePath);
      if (!isVideo) continue;

      try {
        const result = await invoke("pick_video_file_by_path", { path: filePath });

        if (result) {
          /** @type {any} */
          const store = clipsStore;
          store.update((/** @type {any} */ clips) => [
            ...clips,
            {
              id: `clip-${Date.now()}-${Math.random()}`,
              // @ts-ignore
              filename: result.filename,
              // @ts-ignore
              path: result.path,
              // @ts-ignore
              duration: result.duration,
              // @ts-ignore
              resolution: result.resolution,
            },
          ]);
        }
      } catch (err) {
        console.error("Error importing dropped file:", err);
      }
    }
  }

  /**
   * Handle HTML5 drag over event for external file drops
   * @param {DragEvent} e
   */
  function handleExternalDragOver(e) {
    // Check if this is an external file drag (not internal clip drag)
    const types = e.dataTransfer?.types || [];
    const isFileDrag = types.includes('Files');

    if (isFileDrag) {
      e.preventDefault();
      e.stopPropagation();
      isDraggingOver = true;
      if (e.dataTransfer) {
        e.dataTransfer.dropEffect = 'copy';
      }
    }
  }

  /**
   * Handle HTML5 drop event for external files
   * @param {DragEvent} e
   */
  async function handleExternalDrop(e) {
    const types = e.dataTransfer?.types || [];
    const isFileDrag = types.includes('Files');

    if (isFileDrag) {
      e.preventDefault();
      e.stopPropagation();
      isDraggingOver = false;

      const files = e.dataTransfer?.files;
      if (!files) return;

      const paths = [];
      for (let i = 0; i < files.length; i++) {
        const file = files[i];
        // @ts-ignore - File has a path property in Electron/Tauri
        if (file.path) {
          paths.push(file.path);
        }
      }

      if (paths.length > 0) {
        await handleFileDrop(paths);
      }
    }
  }

  function handleDragLeave() {
    isDraggingOver = false;
  }

  /**
   * Workaround: Add clip to timeline at the end
   * TODO: Re-enable drag-drop when Tauri issue is resolved
   * @param {string} clipId
   * @param {number} trackIndex
   */
  function addToTimeline(clipId, trackIndex) {
    const clip = $clipsStore.find(c => c.id === clipId);
    if (!clip) return;

    const timelineClip = {
      id: `timeline-clip-${Date.now()}-${Math.random()}`,
      clipId: clip.id,
      track: trackIndex,
      startTime: $timelineStore.duration, // Add at end
      trimStart: 0,
      trimEnd: clip.duration,
      duration: clip.duration
    };

    console.log('Adding clip to timeline:', timelineClip);

    timelineStore.update(state => ({
      ...state,
      clips: [...state.clips, timelineClip],
      duration: state.duration + clip.duration
    }));
  }
</script>

<div
  class="flex flex-col h-full border-l"
  role="region"
  aria-label="Media library drop zone"
  ondragover={handleExternalDragOver}
  ondrop={handleExternalDrop}
  ondragleave={handleDragLeave}
>
  <div class="px-4 py-3 border-b bg-muted">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-semibold">Media Library</h3>
      <Badge class="" variant="secondary" href="">{$clipsStore.length}</Badge>
    </div>
  </div>

  <ScrollArea class={`flex-1 border-2 border-dashed m-3 rounded-lg transition-colors ${isDraggingOver ? 'border-primary bg-primary/5' : 'border-muted-foreground/30'}`}>
    <div class="p-3 space-y-2">
      {#each $clipsStore as clip (clip.id)}
        <Card
          class={`p-3 cursor-move transition-all hover:shadow-md ${
            $playbackStore.selectedClipId === clip.id
              ? "ring-2 ring-primary bg-accent"
              : "hover:bg-muted"
          }`}
          draggable={true}
          onclick={() => selectClip(clip.id)}
          onkeydown={(/** @type {KeyboardEvent} */ e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              selectClip(clip.id);
            }
          }}
          ondragstart={(/** @type {DragEvent} */ e) => handleDragStart(e, clip)}
          ondragend={() => console.log("Drag ended")}
          role="button"
          tabindex="0"
          data-clip-id={clip.id}
        >
          <div class="flex gap-3 items-center">
            <div
              class="shrink-0 w-10 h-10 bg-muted rounded flex items-center justify-center"
            >
              <Video class="w-5 h-5 text-muted-foreground" />
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate">{clip.filename}</p>
              <p class="text-xs text-muted-foreground">
                {formatTime(clip.duration)} • {clip.resolution}
              </p>
            </div>
            <div class="flex gap-1">
              <Button
                class="shrink-0"
                variant="outline"
                size="icon-sm"
                disabled={false}
                title="Add to Track 1"
                onclick={(/** @type {MouseEvent} */ e) => {
                  e.stopPropagation();
                  addToTimeline(clip.id, 0);
                }}
              >
                <Plus class="w-3 h-3" />
              </Button>
              <Button
                class="shrink-0 bg-destructive hover:bg-destructive/90 text-white rounded-lg cursor-pointer"
                variant="ghost"
                size="icon-sm"
                disabled={false}
                title="Remove clip"
                onclick={(/** @type {MouseEvent} */ e) => {
                  e.stopPropagation();
                  removeClip(clip.id);
                }}
              >
                <X class="w-4 h-4" />
              </Button>
            </div>
          </div>
        </Card>
      {/each}

      {#if $clipsStore.length === 0}
        <div
          class="flex flex-col items-center justify-center py-12 text-center text-muted-foreground"
        >
          <p class="text-sm">No clips imported yet</p>
          <p class="text-xs mt-1">Click Import or drag videos here</p>
        </div>
      {/if}
    </div>
  </ScrollArea>
</div>

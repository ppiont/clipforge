<script>
  import { onMount, onDestroy } from "svelte";
  import { clipsStore } from "../stores/clips.js";
  import { playbackStore } from "../stores/playback.js";
  import { Card } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Button } from "$lib/components/ui/button";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { Video, X } from "@lucide/svelte";

  /**
   * MediaLibrary Component
   * Shows list of imported video clips
   * Allows selection and drag-to-timeline
   */

  let isDraggingOver = $state(false);
  /** @type {(() => void) | null} */
  let unlistenDragDrop = null;

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

  onMount(async () => {
    const webview = getCurrentWebview();

    // Listen for drag drop events
    unlistenDragDrop = await webview.onDragDropEvent((event) => {
      console.log("Drag drop event:", event);

      if (event.payload.type === "drop") {
        // @ts-ignore - Tauri DragDropEvent payload
        const paths = event.payload.paths;
        if (paths && Array.isArray(paths)) {
          handleFileDrop(paths);
        }
        isDraggingOver = false;
      } else if (event.payload.type === "enter" || event.payload.type === "over") {
        isDraggingOver = true;
      } else if (event.payload.type === "leave") {
        isDraggingOver = false;
      }
    });
  });

  onDestroy(() => {
    if (unlistenDragDrop) unlistenDragDrop();
  });
</script>

<div
  class="flex flex-col h-full border-l"
  role="region"
  aria-label="Media library drop zone"
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
          class={`p-3 cursor-pointer transition-all hover:shadow-md ${
            $playbackStore.selectedClipId === clip.id
              ? "ring-2 ring-primary bg-accent"
              : "hover:bg-muted"
          }`}
          draggable="true"
          onclick={() => selectClip(clip.id)}
          onkeydown={(/** @type {KeyboardEvent} */ e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              selectClip(clip.id);
            }
          }}
          ondragstart={(/** @type {DragEvent} */ e) => handleDragStart(e, clip)}
          role="button"
          tabindex="0"
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
            <Button
              class="shrink-0 bg-destructive hover:bg-destructive/90 text-white rounded-lg cursor-pointer"
              variant="ghost"
              size="icon-sm"
              disabled={false}
              onclick={(/** @type {MouseEvent} */ e) => {
                e.stopPropagation();
                removeClip(clip.id);
              }}
            >
              <X class="w-4 h-4" />
            </Button>
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

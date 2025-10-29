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

  /**
   * @typedef {Object} Clip
   * @property {string} id - Unique clip ID
   * @property {string} filename - Original filename
   * @property {string} path - File path
   * @property {number} duration - Duration in seconds
   * @property {string} resolution - Resolution string
   */

  /**
   * @typedef {Object} PlaybackState
   * @property {boolean} isPlaying
   * @property {number} currentTime
   * @property {string | null} selectedClipId
   * @property {string | null} selectedTimelineClipId
   */

  /**
   * @typedef {Object} TimelineClip
   * @property {string} id - Unique timeline clip ID
   * @property {string} clipId - Reference to clips store
   * @property {number} track - 0 = main, 1 = overlay
   * @property {number} startTime - Position on timeline in seconds
   * @property {number} trimStart - Trim in point in seconds
   * @property {number} trimEnd - Trim out point in seconds
   * @property {number} duration - Duration in seconds
   */

  /**
   * @typedef {Object} TimelineState
   * @property {TimelineClip[]} clips - Clips on timeline
   * @property {number} playhead - Current playhead position
   * @property {number} duration - Total timeline duration
   */

  // Type-safe store access
  const clips = /** @type {import('svelte/store').Writable<Clip[]>} */ (
    clipsStore
  );
  const playback =
    /** @type {import('svelte/store').Writable<PlaybackState>} */ (
      playbackStore
    );
  const timeline =
    /** @type {import('svelte/store').Writable<TimelineState>} */ (
      timelineStore
    );

  /**
   * @param {string} clipId
   */
  function selectClip(clipId) {
    playback.update((state) => {
      /** @type {PlaybackState} */
      const newState = {
        ...state,
        // Toggle selection: if already selected, deselect it
        selectedClipId: state.selectedClipId === clipId ? null : clipId,
        selectedTimelineClipId: null,
      };
      return newState;
    });
  }

  /**
   * @param {number} seconds
   * @returns {string}
   */
  function formatTime(seconds) {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  /**
   * @param {DragEvent} e
   * @param {Clip} clip
   */
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

  /**
   * @param {string} clipId
   */
  function removeClip(clipId) {
    clips.update((currentClips) => {
      /** @type {Clip[]} */
      const filtered = currentClips.filter((c) => c.id !== clipId);
      return filtered;
    });

    // If this was the selected clip, clear selection
    if ($playbackStore.selectedClipId === clipId) {
      playback.update((state) => {
        /** @type {PlaybackState} */
        const newState = {
          ...state,
          selectedClipId: null,
        };
        return newState;
      });
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
        const result = await invoke("pick_video_file_by_path", {
          path: filePath,
        });

        if (result && typeof result === "object") {
          const videoData =
            /** @type {{ filename?: string; path?: string; duration?: number; resolution?: string }} */ (
              result
            );

          const clipId = `clip-${Date.now()}-${Math.random()}`;

          /** @type {Clip} */
          const newClip = {
            id: clipId,
            filename: videoData.filename ?? "",
            path: videoData.path ?? filePath,
            duration: videoData.duration ?? 0,
            resolution: videoData.resolution ?? "unknown",
          };

          clips.update((currentClips) => {
            /** @type {Clip[]} */
            const updated = [...currentClips, newClip];
            return updated;
          });

          // Generate thumbnail asynchronously (at 1 second into video)
          try {
            const thumbnailTimestamp = Math.min(1.0, videoData.duration ?? 1.0);
            console.log(`Generating thumbnail for ${filePath} at ${thumbnailTimestamp}s`);

            const thumbnail = await invoke("generate_thumbnail", {
              videoPath: filePath,
              timestamp: thumbnailTimestamp
            });

            console.log(`Thumbnail generated successfully for ${clipId}`);

            // Update the clip with thumbnail
            clips.update((currentClips) => {
              return currentClips.map(c =>
                c.id === clipId ? { ...c, thumbnail: String(thumbnail) } : c
              );
            });
          } catch (err) {
            console.error("Error generating thumbnail for", filePath, ":", err);
            // Continue without thumbnail
          }
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
    const isFileDrag = types.includes("Files");

    if (isFileDrag) {
      e.preventDefault();
      e.stopPropagation();
      isDraggingOver = true;
      if (e.dataTransfer) {
        e.dataTransfer.dropEffect = "copy";
      }
    }
  }

  /**
   * Handle HTML5 drop event for external files
   * @param {DragEvent} e
   */
  async function handleExternalDrop(e) {
    const types = e.dataTransfer?.types || [];
    const isFileDrag = types.includes("Files");

    if (isFileDrag) {
      e.preventDefault();
      e.stopPropagation();
      isDraggingOver = false;

      const files = e.dataTransfer?.files;
      if (!files) return;

      const paths = [];
      for (let i = 0; i < files.length; i++) {
        const file = files.item(i);
        if (!file) continue;

        // Tauri/Electron adds path property to File objects
        const fileWithPath = file;
        if (typeof fileWithPath === "object" && "path" in fileWithPath) {
          const filePath = fileWithPath["path"];
          if (typeof filePath === "string") {
            paths.push(filePath);
          }
        }
      }

      if (paths.length > 0) {
        await handleFileDrop(paths);
      }
    }
  }

  /**
   * Handle drag leave event
   */
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
    const clip = $clipsStore.find((c) => c.id === clipId);
    if (!clip) return;

    /** @type {TimelineClip} */
    const timelineClip = {
      id: `timeline-clip-${Date.now()}-${Math.random()}`,
      clipId: clip.id,
      track: trackIndex,
      startTime: $timelineStore.duration, // Add at end
      trimStart: 0,
      trimEnd: clip.duration,
      duration: clip.duration,
    };

    console.log("Adding clip to timeline:", timelineClip);

    timeline.update((state) => {
      /** @type {TimelineClip[]} */
      const updatedClips = [...state.clips, timelineClip];
      /** @type {TimelineState} */
      const newState = {
        ...state,
        clips: updatedClips,
        duration: state.duration + clip.duration,
      };
      return newState;
    });
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
      <Badge variant="secondary">{$clipsStore.length}</Badge>
    </div>
  </div>

  <ScrollArea
    class={`flex-1 border-2 border-dashed m-3 rounded-lg transition-colors ${isDraggingOver ? "border-primary bg-primary/5" : "border-muted-foreground/30"}`}
  >
    <div class="p-3 space-y-2">
      {#each $clipsStore as clip (clip.id)}
        <Card
          class={`p-3 cursor-move transition-all duration-200 active:scale-95 ${
            $playbackStore.selectedClipId === clip.id
              ? "ring-[3px] ring-primary bg-accent shadow-md"
              : "hover:bg-muted hover:shadow-lg hover:-translate-y-0.5"
          }`}
          draggable={true}
          onclick={() => selectClip(clip.id)}
          onkeydown={(/** @type {KeyboardEvent} */ e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              selectClip(clip.id);
            }
          }}
          ondragstart={(/** @type {DragEvent} */ e) => {
            handleDragStart(e, clip);
          }}
          ondragend={() => console.log("Drag ended")}
          role="button"
          tabindex="0"
          data-clip-id={clip.id}
        >
          <div class="flex gap-3 items-center">
            <div
              class="shrink-0 w-20 h-[45px] bg-muted rounded flex items-center justify-center overflow-hidden"
            >
              {#if clip.thumbnail}
                <img
                  src={clip.thumbnail}
                  alt={clip.filename}
                  class="w-full h-full object-cover"
                />
              {:else}
                <Video class="w-5 h-5 text-muted-foreground" />
              {/if}
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate" title={clip.filename}>{clip.filename}</p>
              <p class="text-xs text-muted-foreground">
                {formatTime(clip.duration)} • {clip.resolution}
              </p>
            </div>
            <div class="flex gap-1">
              <Button
                variant="outline"
                size="icon-sm"
                title="Add to Track 1"
                onclick={(/** @type {MouseEvent} */ e) => {
                  e.stopPropagation();
                  addToTimeline(clip.id, 0);
                }}
              >
                <Plus />
              </Button>
              <Button
                variant="destructive"
                size="icon-sm"
                title="Remove clip"
                onclick={(/** @type {MouseEvent} */ e) => {
                  e.stopPropagation();
                  removeClip(clip.id);
                }}
              >
                <X />
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

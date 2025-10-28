<script>
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
    DialogFooter
  } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress";
  import { invoke } from "@tauri-apps/api/core";
  import { save } from "@tauri-apps/plugin-dialog";
  import { timelineStore } from "../stores/timeline.js";
  import { clipsStore } from "../stores/clips.js";

  /**
   * @typedef {import('../stores/timeline.js').TimelineClip} TimelineClip
   * @typedef {import('../stores/clips.js').Clip} Clip
   */

  /**
   * ExportModal Component
   * Modal dialog for exporting video
   */

  let {
    show = $bindable(false),
    onClose = () => {}
  } = $props();

  let resolution = $state("Source");
  let isExporting = $state(false);
  let progress = $state(0);
  let errorMessage = $state("");

  /** @param {boolean} _value */
  function handleOpenChange(_value) {
    if (!isExporting) {
      errorMessage = "";
      onClose();
    }
  }

  /** @param {string} value */
  function updateResolution(value) {
    resolution = value;
  }

  async function handleExport() {
    if ($timelineStore.clips.length === 0) {
      errorMessage = "No clips on timeline to export";
      return;
    }

    try {
      // Show save dialog
      const outputPath = await save({
        filters: [{
          name: "Video",
          extensions: ["mp4"]
        }],
        defaultPath: "export.mp4"
      });

      if (!outputPath) {
        return; // User cancelled
      }

      isExporting = true;
      progress = 0;
      errorMessage = "";

      // Create a map of clip IDs to their data
      /** @type {Record<string, {filename: string, path: string, duration: number, resolution: string, codec: string}>} */
      const clipMap = {};
      $clipsStore.forEach(c => {
        clipMap[c.id] = {
          filename: c.filename,
          path: c.path,
          duration: c.duration,
          resolution: c.resolution,
          codec: c.codec ?? "unknown"
        };
      });

      // Prepare export request with resolved paths
      const exportRequest = {
        clips: $timelineStore.clips.map((/** @type {TimelineClip} */ c) => {
          const sourceClip = clipMap[c.clipId];
          if (!sourceClip) {
            throw new Error(`Clip ${c.clipId} not found in media library`);
          }
          return {
            id: c.id,
            clip_id: sourceClip.path, // Send the actual file path
            track: c.track,
            start_time: c.startTime,
            trim_start: c.trimStart,
            trim_end: c.trimEnd,
            duration: c.duration
          };
        }),
        output_path: outputPath,
        resolution: resolution
      };

      // Get clip metadata
      const clipsData = Object.values(clipMap);

      console.log("Exporting:", exportRequest);

      // Call Rust backend
      const result = await invoke("export_video", {
        request: exportRequest,
        clipsData: clipsData
      });

      progress = 100;
      console.log("Export successful:", result);

      // Close dialog after brief delay
      setTimeout(() => {
        isExporting = false;
        show = false;
        onClose();
      }, 1000);

    } catch (err) {
      console.error("Export error:", err);
      errorMessage = err?.toString() || "Export failed";
      isExporting = false;
      progress = 0;
    }
  }
</script>

<Dialog open={show} onOpenChange={handleOpenChange}>
  <DialogContent class="sm:max-w-[500px]" portalProps={{}}>
    <DialogHeader class="">
      <DialogTitle class="">Export Video</DialogTitle>
      <DialogDescription class="">
        Configure and export your video composition
      </DialogDescription>
    </DialogHeader>

    <div class="space-y-4 py-4">
      {#if errorMessage}
        <div class="p-3 bg-destructive/10 border border-destructive rounded text-sm text-destructive">
          {errorMessage}
        </div>
      {/if}

      {#if !isExporting}
        <div class="space-y-2">
          <label for="resolution" class="text-sm font-medium">Resolution</label>
          <div class="space-y-2">
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="resolution"
                value="Source"
                checked={resolution === "Source"}
                onchange={(e) => updateResolution(e.currentTarget.value)}
                class="cursor-pointer"
              />
              <span>Source (Original)</span>
            </label>
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="resolution"
                value="720p"
                checked={resolution === "720p"}
                onchange={(e) => updateResolution(e.currentTarget.value)}
                class="cursor-pointer"
              />
              <span>720p (1280x720)</span>
            </label>
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="resolution"
                value="1080p"
                checked={resolution === "1080p"}
                onchange={(e) => updateResolution(e.currentTarget.value)}
                class="cursor-pointer"
              />
              <span>1080p (1920x1080)</span>
            </label>
          </div>
        </div>
      {:else}
        <div class="space-y-2">
          <div class="flex items-center justify-between text-sm">
            <span>Exporting...</span>
            <span class="font-medium">{progress}%</span>
          </div>
          <Progress value={progress} class="w-full" />
        </div>
      {/if}
    </div>

    <DialogFooter class="">
      <Button variant="outline" disabled={isExporting} class="" onclick={handleOpenChange}>
        Cancel
      </Button>
      <Button disabled={isExporting} class="" onclick={handleExport}>
        {isExporting ? 'Exporting...' : 'Export'}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

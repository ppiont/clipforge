<script>
  // @ts-nocheck - shadcn-svelte components have overly strict type definitions
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
    DialogFooter,
  } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress";
  import {
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
  } from "$lib/components/ui/tabs";
  import * as Select from "$lib/components/ui/select/index.js";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { Separator } from "$lib/components/ui/separator";
  import { FileVideoCamera, Monitor, Settings } from "@lucide/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { save } from "@tauri-apps/plugin-dialog";
  import { timelineStore } from "../stores/timeline.js";
  import { clipsStore } from "../stores/clips.js";
  import {
    EXPORT_RESOLUTIONS,
    EXPORT_FORMATS,
    estimateFileSize,
  } from "../config/export.js";

  /**
   * ExportModal Component
   * Modern modal dialog for exporting video with configuration options
   */

  let { show = $bindable(false), onClose = () => {} } = $props();

  let resolution = $state("1080p");
  let format = $state("mp4");
  let isExporting = $state(false);
  let progress = $state(0);
  let errorMessage = $state("");
  let activeTab = $state("video");

  // Computed values
  const selectedResolution = $derived(
    EXPORT_RESOLUTIONS.find((r) => r.value === resolution) ||
      EXPORT_RESOLUTIONS[2],
  );
  const selectedFormat = $derived(
    EXPORT_FORMATS.find((f) => f.value === format) || EXPORT_FORMATS[0],
  );

  // Calculate total timeline duration
  const totalDuration = $derived(
    $timelineStore.clips.reduce((sum, clip) => sum + (/** @type {any} */ (clip).duration || 0), 0),
  );

  // Estimate file size
  const estimatedSize = $derived(
    estimateFileSize(resolution, totalDuration, format),
  );

  /** @param {boolean} value */
  function handleOpenChange(value) {
    if (!isExporting) {
      show = value;
      if (!value) {
        // Resetting state when closing
        errorMessage = "";
        activeTab = "video";
        onClose();
      }
    }
  }

  async function handleExport() {
    if ($timelineStore.clips.length === 0) {
      errorMessage = "No clips on timeline to export";
      return;
    }

    // Variable to hold the unlisten function
    let unlisten = null;

    try {
      // Show save dialog with appropriate extension
      const outputPath = await save({
        filters: [
          {
            name: "Video",
            extensions: [format],
          },
        ],
        defaultPath: `export.${format}`,
      });

      if (!outputPath) {
        return; // User cancelled
      }

      isExporting = true;
      progress = 0;
      errorMessage = "";

      // Set up progress event listener before starting export
      unlisten = await listen("export_progress", (event) => {
        progress = event.payload;
        console.log("Export progress:", progress);
      });

      // Create a map of clip IDs to their data
      /** @type {Record<string, {filename: string, path: string, duration: number, resolution: string, codec: string}>} */
      const clipMap = {};
      $clipsStore.forEach((c) => {
        clipMap[c.id] = {
          filename: c.filename,
          path: c.path,
          duration: c.duration,
          resolution: c.resolution,
          codec: c.codec ?? "unknown",
        };
      });

      // Prepare export request with resolved paths
      const exportRequest = {
        clips: $timelineStore.clips.map((c) => {
          const clip = /** @type {any} */ (c);
          const sourceClip = clipMap[clip.clipId];
          if (!sourceClip) {
            throw new Error(`Clip ${clip.clipId} not found in media library`);
          }
          return {
            id: clip.id,
            clip_id: sourceClip.path, // Send the actual file path
            track: clip.track,
            start_time: clip.startTime,
            trim_start: clip.trimStart,
            trim_end: clip.trimEnd,
            duration: clip.duration,
          };
        }),
        output_path: outputPath,
        resolution: resolution,
        format: format,
      };

      // Get clip metadata
      const clipsData = Object.values(clipMap);

      console.log("Exporting:", exportRequest);

      // Call Rust backend
      const result = await invoke("export_video", {
        request: exportRequest,
        clipsData: clipsData,
      });

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
    } finally {
      // Clean up event listener
      if (unlisten) {
        unlisten();
      }
    }
  }
</script>

<Dialog open={show} onOpenChange={handleOpenChange}>
  <DialogContent class="sm:max-w-[550px]">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <FileVideoCamera />
        Export Video
      </DialogTitle>
      <DialogDescription>
        Configure your video export settings and format
      </DialogDescription>
    </DialogHeader>

    <div class="space-y-4">
      {#if errorMessage}
        <div
          class="p-3 bg-destructive/10 border border-destructive rounded-md text-sm text-destructive"
        >
          {errorMessage}
        </div>
      {/if}

      {#if !isExporting}
        <Tabs bind:value={activeTab} class="w-full">
          <TabsList class="grid w-full grid-cols-2">
            <TabsTrigger value="video" class="flex items-center gap-2">
              <Monitor />
              Video
            </TabsTrigger>
            <TabsTrigger value="advanced" class="flex items-center gap-2">
              <Settings />
              Advanced
            </TabsTrigger>
          </TabsList>

          <TabsContent value="video" class="space-y-4 mt-4">
            <Card>
              <CardHeader>
                <CardTitle class="text-base">Output Settings</CardTitle>
                <CardDescription>Choose resolution and format</CardDescription>
              </CardHeader>
              <CardContent class="space-y-4">
                <!-- Format Selection -->
                <div class="space-y-2">
                  <span class="text-sm font-medium">Format</span>
                  <Select.Root type="single" bind:value={format}>
                    <Select.Trigger class="w-[200px]">
                      <span>{selectedFormat.label}</span>
                    </Select.Trigger>
                    <Select.Content class="w-[200px]">
                      <Select.Group>
                        {#each EXPORT_FORMATS as fmt (fmt.value)}
                          <Select.Item value={fmt.value} label={fmt.label} />
                        {/each}
                      </Select.Group>
                    </Select.Content>
                  </Select.Root>
                  <p class="text-xs text-muted-foreground">
                    {selectedFormat.description}
                  </p>
                </div>

                <!-- Resolution Selection -->
                <div class="space-y-2">
                  <span class="text-sm font-medium">Resolution</span>
                  <Select.Root type="single" bind:value={resolution}>
                    <Select.Trigger class="w-[200px]">
                      <span>{selectedResolution.label}</span>
                    </Select.Trigger>
                    <Select.Content class="w-[200px]">
                      <Select.Group>
                        {#each EXPORT_RESOLUTIONS as res (res.value)}
                          <Select.Item value={res.value} label={res.label} />
                        {/each}
                      </Select.Group>
                    </Select.Content>
                  </Select.Root>
                  <p class="text-xs text-muted-foreground">
                    {selectedResolution.description}
                  </p>
                </div>
              </CardContent>
            </Card>

            <Separator />

            <!-- Export Preview -->
            <div class="space-y-2">
              <p class="text-sm font-medium">Export Summary</p>
              <div class="flex flex-wrap gap-2">
                <Badge variant="secondary">
                  {selectedResolution.description || "Original"}
                </Badge>
                <Badge variant="secondary">
                  {selectedFormat.label}
                </Badge>
                <Badge variant="outline">
                  {Math.floor(totalDuration / 60)}:{String(
                    Math.floor(totalDuration % 60),
                  ).padStart(2, "0")} duration
                </Badge>
                <Badge variant="outline">
                  ~{estimatedSize}
                </Badge>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="advanced" class="space-y-4 mt-4">
            <Card>
              <CardHeader>
                <CardTitle class="text-base">Advanced Settings</CardTitle>
                <CardDescription>Additional export options</CardDescription>
              </CardHeader>
              <CardContent class="space-y-4">
                <p class="text-sm text-muted-foreground">
                  Advanced settings coming soon. Future options will include:
                </p>
                <ul
                  class="text-sm text-muted-foreground space-y-1 list-disc list-inside"
                >
                  <li>Custom bitrate control</li>
                  <li>Frame rate adjustment</li>
                  <li>Audio quality settings</li>
                  <li>Codec selection</li>
                </ul>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      {:else}
        <!-- Export Progress -->
        <Card>
          <CardContent class="pt-6">
            <div class="space-y-4">
              <div class="flex items-center justify-between text-sm">
                <span class="font-medium">Exporting video...</span>
                <span class="font-bold text-primary">{progress}%</span>
              </div>
              <Progress value={progress} class="w-full h-2" />
              <p class="text-xs text-muted-foreground text-center">
                This may take a few moments depending on video length
              </p>
            </div>
          </CardContent>
        </Card>
      {/if}
    </div>

    <DialogFooter>
      <Button
        variant="outline"
        disabled={isExporting}
        class="active:scale-95 transition-transform"
        onclick={() => handleOpenChange(false)}
      >
        Cancel
      </Button>
      <Button disabled={isExporting} class="active:scale-95 transition-transform" onclick={handleExport}>
        {isExporting ? "Exporting..." : "Export Video"}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

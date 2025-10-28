<script>
  /**
   * Editor View
   * Main editor window layout combining all components
   */

  import TopBar from '../components/TopBar.svelte';
  import Preview from '../components/Preview.svelte';
  import MediaLibrary from '../components/MediaLibrary.svelte';
  import Timeline from '../components/Timeline.svelte';
  import Controls from '../components/Controls.svelte';
  import ExportModal from '../components/ExportModal.svelte';
  import { clipsStore } from '../stores/clips.js';
  import { invoke } from '@tauri-apps/api/core';

  let videoElement = $state(null);
  let showExportModal = $state(false);

  async function handleImportClick() {
    try {
      const result = await invoke('pick_video_file');
      if (result) {
        console.log('Imported video:', result);

        const clipId = `clip-${Date.now()}`;

        // Add to clipsStore
        clipsStore.update(clips => [
          ...clips,
          {
            id: clipId,
            filename: result.filename,
            path: result.path,
            duration: result.duration,
            resolution: result.resolution
          }
        ]);

        // Generate thumbnail asynchronously (at 1 second into video)
        try {
          const thumbnailTimestamp = Math.min(1.0, result.duration ?? 1.0);
          console.log(`Generating thumbnail for ${result.path} at ${thumbnailTimestamp}s`);

          const thumbnail = await invoke("generate_thumbnail", {
            videoPath: result.path,
            timestamp: thumbnailTimestamp
          });

          console.log(`Thumbnail generated successfully for ${clipId}`);

          // Update the clip with thumbnail
          clipsStore.update(clips => {
            return clips.map(c =>
              c.id === clipId ? { ...c, thumbnail: String(thumbnail) } : c
            );
          });
        } catch (err) {
          console.error("Error generating thumbnail for", result.path, ":", err);
          // Continue without thumbnail
        }
      }
    } catch (err) {
      console.error('Error picking file:', err);
    }
  }

  function handleRecordClick() {
    // Task 8.1: Open recorder window
    console.log('Record button clicked (coming in Task 8.1)');
  }

  function handleExportClick() {
    showExportModal = true;
  }
</script>

<div class="flex flex-col w-full h-full bg-background">
  <TopBar
    onImportClick={handleImportClick}
    onRecordClick={handleRecordClick}
    onExportClick={handleExportClick}
  />

  <div class="flex flex-1 overflow-hidden">
    <div class="flex-1 flex items-center justify-center bg-black p-2">
      <Preview bind:videoElement={videoElement} />
    </div>
    <div class="flex-shrink-0 w-[300px] flex flex-col">
      <MediaLibrary />
    </div>
  </div>

  <Timeline {videoElement} />

  <Controls {videoElement} />

  <ExportModal bind:show={showExportModal} />
</div>

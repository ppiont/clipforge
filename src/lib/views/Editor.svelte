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
  import { invoke } from '@tauri-apps/api/core';

  /** @type {HTMLVideoElement | null} */
  let videoElement = null;
  let showExportModal = false;

  async function handleImportClick() {
    try {
      const result = await invoke('pick_video_file');
      if (result) {
        console.log('Selected file:', result);
        // Task 2.1-2.2: Handle video import
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

  <Timeline />

  <Controls {videoElement} />

  <ExportModal bind:show={showExportModal} />
</div>

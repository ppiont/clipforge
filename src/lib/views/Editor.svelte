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

<div class="editor-container">
  <TopBar
    onImportClick={handleImportClick}
    onRecordClick={handleRecordClick}
    onExportClick={handleExportClick}
  />

  <div class="main-area">
    <div class="preview-section">
      <Preview bind:videoElement={videoElement} />
    </div>
    <div class="library-section">
      <MediaLibrary />
    </div>
  </div>

  <Timeline />

  <Controls {videoElement} />

  <ExportModal bind:show={showExportModal} />
</div>

<style>
  .editor-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    background: #fff;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
      'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
  }

  .main-area {
    display: flex;
    flex: 1;
    min-height: 600px;
    overflow: hidden;
  }

  .preview-section {
    flex: 2;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #000;
    padding: 10px;
  }

  .library-section {
    flex: 1;
    min-width: 250px;
    display: flex;
    flex-direction: column;
    background: #fafafa;
  }
</style>

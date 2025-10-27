<script>
  import { clipsStore } from '../stores/clips.js';
  import { playbackStore } from '../stores/playback.js';

  /**
   * MediaLibrary Component
   * Shows list of imported video clips
   * Allows selection and drag-to-timeline
   */

  /** @param {string} clipId */
  function selectClip(clipId) {
    playbackStore.update(state => ({
      ...state,
      selectedClipId: clipId,
      selectedTimelineClipId: null // Deselect timeline clip
    }));
  }

  /** @param {number} seconds */
  function formatTime(seconds) {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  /** @param {DragEvent} e
   *  @param {{id: string, duration: number, path: string}} clip */
  function handleDragStart(e, clip) {
    if (!e.dataTransfer) return;
    e.dataTransfer.effectAllowed = 'copy';
    e.dataTransfer.setData('application/json', JSON.stringify({
      clipId: clip.id,
      duration: clip.duration,
      path: clip.path
    }));
  }
</script>

<div class="media-library">
  <div class="header">
    <h3>Media Library</h3>
    <span class="count">{$clipsStore.length}</span>
  </div>

  <div class="clips-container">
    {#each $clipsStore as clip (clip.id)}
      <div
        class="clip-item"
        class:selected={$playbackStore.selectedClipId === clip.id}
        draggable="true"
        on:click={() => selectClip(clip.id)}
        on:dragstart={(e) => handleDragStart(e, clip)}
      >
        <div class="clip-thumbnail">
          <div class="video-icon">🎬</div>
        </div>
        <div class="clip-info">
          <div class="filename">{clip.filename}</div>
          <div class="metadata">
            {formatTime(clip.duration)} • {clip.resolution}
          </div>
        </div>
      </div>
    {/each}

    {#if $clipsStore.length === 0}
      <div class="empty-state">
        <p>No clips imported yet</p>
        <p style="font-size: 12px; color: #999;">Click Import to add videos</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .media-library {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #fafafa;
    border-left: 1px solid #ddd;
    overflow: hidden;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 15px;
    border-bottom: 1px solid #ddd;
    background: #f5f5f5;
  }

  .header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: #333;
  }

  .count {
    font-size: 12px;
    color: #999;
    background: #e0e0e0;
    padding: 2px 8px;
    border-radius: 12px;
  }

  .clips-container {
    flex: 1;
    overflow-y: auto;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .clip-item {
    display: flex;
    gap: 10px;
    padding: 10px;
    background: #fff;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
    user-select: none;
  }

  .clip-item:hover {
    background: #f8f8f8;
    border-color: #999;
  }

  .clip-item.selected {
    background: #e3f2fd;
    border-color: #1976d2;
    box-shadow: 0 0 0 2px rgba(25, 118, 210, 0.1);
  }

  .clip-thumbnail {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    background: #e0e0e0;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .video-icon {
    font-size: 20px;
  }

  .clip-info {
    display: flex;
    flex-direction: column;
    justify-content: center;
    flex: 1;
    min-width: 0;
  }

  .filename {
    font-size: 13px;
    font-weight: 500;
    color: #333;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .metadata {
    font-size: 11px;
    color: #999;
    margin-top: 2px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #999;
    text-align: center;
  }

  .empty-state p {
    margin: 5px 0;
  }
</style>

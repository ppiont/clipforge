<script>
  /**
   * TopBar Component
   * Contains Import, Record, and Export buttons
   */

  export let onImportClick = () => {};
  export let onRecordClick = () => {};
  export let onExportClick = () => {};
</script>

<div class="top-bar">
  <div class="title">ClipForge</div>
  <div class="button-group">
    <button on:click={onImportClick}>📁 Import</button>
    <button on:click={onRecordClick}>⭕ Record</button>
    <button on:click={onExportClick}>💾 Export</button>
  </div>
</div>

<style>
  .top-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    height: 50px;
    padding: 0 15px;
    background: #f5f5f5;
    border-bottom: 1px solid #ddd;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .title {
    font-size: 18px;
    font-weight: 600;
    color: #333;
  }

  .button-group {
    display: flex;
    gap: 10px;
  }

  button {
    padding: 8px 16px;
    background: #fff;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    transition: all 0.2s;
  }

  button:hover {
    background: #f0f0f0;
    border-color: #999;
  }

  button:active {
    background: #e0e0e0;
  }
</style>

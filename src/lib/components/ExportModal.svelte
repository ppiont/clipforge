<script>
  /**
   * ExportModal Component
   * Modal dialog for exporting video
   * (Stub for Task 6.2)
   */

  export let show = false;
  export let onClose = () => {};
</script>

{#if show}
  <div class="modal-overlay" on:click={onClose}>
    <div class="modal" on:click|stopPropagation>
      <div class="modal-header">
        <h2>Export Video</h2>
        <button class="close-btn" on:click={onClose}>✕</button>
      </div>

      <div class="modal-content">
        <p style="color: #999; text-align: center;">
          Export functionality coming soon (Task 6.2)
        </p>
      </div>

      <div class="modal-footer">
        <button class="cancel-btn" on:click={onClose}>Cancel</button>
        <button class="primary-btn" disabled>Export</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: white;
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
    width: 90%;
    max-width: 500px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px;
    border-bottom: 1px solid #ddd;
    background: #f5f5f5;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 18px;
    color: #333;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 20px;
    color: #666;
    cursor: pointer;
    padding: 0;
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .close-btn:hover {
    color: #333;
  }

  .modal-content {
    padding: 30px;
    flex: 1;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    padding: 15px 20px;
    border-top: 1px solid #ddd;
    background: #f5f5f5;
  }

  button {
    padding: 8px 16px;
    border-radius: 4px;
    border: none;
    cursor: pointer;
    font-size: 14px;
    transition: all 0.2s;
  }

  .cancel-btn {
    background: #fff;
    border: 1px solid #ddd;
    color: #333;
  }

  .cancel-btn:hover {
    background: #f0f0f0;
  }

  .primary-btn {
    background: #1976d2;
    color: white;
  }

  .primary-btn:hover:not(:disabled) {
    background: #1565c0;
  }

  .primary-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>

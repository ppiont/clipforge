<script>
  /**
   * Recorder View
   * Floating recorder window (400x300px)
   * (Stub for Task 8.1)
   */

  import { recordingStore } from '../stores/recording.js';
</script>

<div class="recorder-container">
  <div class="status-bar">
    <div class="recording-indicator" class:active={$recordingStore.isRecording}>
      <span class="dot"></span>
      {$recordingStore.isRecording ? 'Recording...' : 'Ready'}
    </div>
    <div class="timer">
      {Math.floor($recordingStore.recordedDuration / 60)}:
      {Math.floor($recordingStore.recordedDuration % 60)
        .toString()
        .padStart(2, '0')}
    </div>
  </div>

  <div class="preview-area">
    <p style="color: #999; text-align: center; margin-top: 70px;">
      Preview will appear here
    </p>
  </div>

  <div class="source-selector">
    <label>
      <input type="radio" name="source" value="screen" disabled />
      Screen
    </label>
    <label>
      <input type="radio" name="source" value="webcam" disabled />
      Webcam
    </label>
    <label>
      <input type="radio" name="source" value="both" disabled />
      Both
    </label>
  </div>

  <div class="action-buttons">
    <button disabled>Stop</button>
    <button disabled>Cancel</button>
  </div>
</div>

<style>
  .recorder-container {
    display: flex;
    flex-direction: column;
    width: 400px;
    height: 300px;
    background: #222;
    color: #fff;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
      'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
  }

  .status-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 15px;
    background: #1a1a1a;
    border-bottom: 1px solid #333;
    height: 40px;
  }

  .recording-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: #999;
  }

  .recording-indicator.active {
    color: #ff4444;
  }

  .dot {
    width: 8px;
    height: 8px;
    background: #ff4444;
    border-radius: 50%;
    animation: pulse 1s infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .timer {
    font-size: 13px;
    color: #fff;
    font-family: monospace;
  }

  .preview-area {
    flex: 1;
    background: #000;
    border-bottom: 1px solid #333;
  }

  .source-selector {
    display: flex;
    gap: 15px;
    padding: 10px 15px;
    background: #1a1a1a;
    border-bottom: 1px solid #333;
    font-size: 12px;
  }

  .source-selector label {
    display: flex;
    align-items: center;
    gap: 5px;
    cursor: pointer;
  }

  .source-selector input {
    cursor: pointer;
  }

  .action-buttons {
    display: flex;
    gap: 10px;
    padding: 10px;
    background: #1a1a1a;
  }

  .action-buttons button {
    flex: 1;
    padding: 8px;
    background: #333;
    color: #fff;
    border: 1px solid #555;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    transition: all 0.2s;
  }

  .action-buttons button:hover:not(:disabled) {
    background: #444;
    border-color: #777;
  }

  .action-buttons button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>

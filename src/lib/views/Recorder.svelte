<script>
  /**
   * Recorder View
   * Floating recorder window (400x300px)
   * Implements screen + webcam PiP recording with canvas compositing
   */

  import { invoke } from '@tauri-apps/api/core';
  import { onDestroy } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import * as Alert from '$lib/components/ui/alert';
  import { recordingStore } from '../stores/recording.js';
  import { clipsStore, generateFilmstripForClip } from '../stores/clips.js';
  import { Circle, X } from '@lucide/svelte';

  // Recording state
  let isRecording = $state(false);
  let recordedDuration = $state(0);
  let timerInterval = null;
  let error = $state('');

  // Media elements and streams
  let screenStream = null;
  let webcamStream = null;
  let compositeStream = null;
  let mediaRecorder = null;
  let recordedChunks = [];

  // Canvas references
  let canvas = $state(null);
  let ctx = null;
  let screenVideo = null;
  let webcamVideo = null;
  let webcamPreviewElement = null;

  // Animation frame ID
  let animationFrameId = null;

  /**
   * Start recording: Acquire streams and set up canvas compositing
   */
  async function startRecording() {
    try {
      error = '';
      console.log('Starting PiP recording...');

      // Step 1: Get screen stream
      screenStream = await navigator.mediaDevices.getDisplayMedia({
        video: {
          cursor: 'always',
          displaySurface: 'monitor'
        },
        audio: {
          echoCancellation: true,
          noiseSuppression: true
        }
      });
      console.log('Screen stream acquired');

      // Step 2: Get webcam stream
      webcamStream = await navigator.mediaDevices.getUserMedia({
        video: {
          width: { ideal: 1280 },
          height: { ideal: 720 }
        },
        audio: false // Use screen audio only
      });
      console.log('Webcam stream acquired');

      // Step 3: Set up canvas for compositing
      setupCanvas();

      // Step 4: Start MediaRecorder with composite stream
      startMediaRecorder();

      // Step 5: Update UI state
      isRecording = true;
      recordedDuration = 0;
      timerInterval = setInterval(() => {
        recordedDuration += 1;
      }, 1000);

      // Update recording store
      recordingStore.update(state => ({
        ...state,
        isRecording: true,
        source: 'both',
        startTime: Date.now(),
        recordedDuration: 0
      }));

      console.log('Recording started successfully');
    } catch (err) {
      console.error('Error starting recording:', err);
      error = `Failed to start recording: ${err.message || err}`;
      cleanup();
    }
  }

  /**
   * Set up canvas for PiP compositing
   */
  function setupCanvas() {
    if (!canvas) return;

    // Set canvas size
    canvas.width = 1920;
    canvas.height = 1080;
    ctx = canvas.getContext('2d');

    // Create video elements for streams
    screenVideo = document.createElement('video');
    webcamVideo = document.createElement('video');

    screenVideo.srcObject = screenStream;
    webcamVideo.srcObject = webcamStream;

    // Important: muted and autoplay required for video elements
    screenVideo.muted = true;
    screenVideo.autoplay = true;
    webcamVideo.muted = true;
    webcamVideo.autoplay = true;

    // Show webcam preview in UI
    if (webcamPreviewElement) {
      webcamPreviewElement.srcObject = webcamStream;
      webcamPreviewElement.play().catch(console.error);
    }

    // Wait for videos to be ready, then start drawing
    Promise.all([
      new Promise(resolve => {
        screenVideo.onloadedmetadata = resolve;
      }),
      new Promise(resolve => {
        webcamVideo.onloadedmetadata = resolve;
      })
    ]).then(() => {
      screenVideo.play();
      webcamVideo.play();
      drawFrame();
    });
  }

  /**
   * Draw composite frame: screen with webcam PiP overlay
   */
  function drawFrame() {
    if (!ctx || !isRecording) return;

    // Draw screen video to full canvas
    ctx.drawImage(screenVideo, 0, 0, canvas.width, canvas.height);

    // Draw webcam as PiP in bottom-right corner
    const pipWidth = 320;
    const pipHeight = 240;
    const pipX = canvas.width - pipWidth - 20;
    const pipY = canvas.height - pipHeight - 20;

    ctx.drawImage(webcamVideo, pipX, pipY, pipWidth, pipHeight);

    // Draw border around PiP
    ctx.strokeStyle = '#fff';
    ctx.lineWidth = 3;
    ctx.strokeRect(pipX, pipY, pipWidth, pipHeight);

    // Continue animation loop
    animationFrameId = requestAnimationFrame(drawFrame);
  }

  /**
   * Start MediaRecorder with composite stream
   */
  function startMediaRecorder() {
    // Capture composite stream from canvas
    compositeStream = canvas.captureStream(30);

    // Add audio track from screen stream
    const audioTrack = screenStream.getAudioTracks()[0];
    if (audioTrack) {
      compositeStream.addTrack(audioTrack);
    }

    // Create MediaRecorder
    recordedChunks = [];
    mediaRecorder = new MediaRecorder(compositeStream, {
      mimeType: 'video/webm;codecs=vp9',
      videoBitsPerSecond: 2500000
    });

    mediaRecorder.ondataavailable = (event) => {
      if (event.data.size > 0) {
        recordedChunks.push(event.data);
      }
    };

    mediaRecorder.onstop = async () => {
      console.log('MediaRecorder stopped, saving recording...');
      await saveRecording();
    };

    mediaRecorder.start();
    console.log('MediaRecorder started');
  }

  /**
   * Stop recording
   */
  function stopRecording() {
    console.log('Stopping recording...');
    if (mediaRecorder && mediaRecorder.state !== 'inactive') {
      mediaRecorder.stop();
    }
    cleanup();
  }

  /**
   * Cancel recording (discard without saving)
   */
  function cancelRecording() {
    console.log('Canceling recording...');
    recordedChunks = []; // Discard chunks
    cleanup();
    closeWindow();
  }

  /**
   * Save recording to disk and auto-import
   */
  async function saveRecording() {
    try {
      if (recordedChunks.length === 0) {
        throw new Error('No recorded data');
      }

      // Create blob from chunks
      const blob = new Blob(recordedChunks, { type: 'video/webm' });
      console.log(`Recording blob size: ${blob.size} bytes`);

      // Convert to array buffer
      const arrayBuffer = await blob.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);

      // Generate filename
      const now = new Date();
      const timestamp = now.toISOString().replace(/[:.]/g, '-').split('T');
      const filename = `recording_${timestamp[0]}_${timestamp[1].slice(0, 8)}.webm`;

      // Save via Tauri command
      console.log(`Saving recording as ${filename}...`);
      const filePath = await invoke('save_recording', {
        blob: Array.from(uint8Array),
        filename
      });
      console.log(`Recording saved to: ${filePath}`);

      // Auto-import to timeline
      await autoImportRecording(filePath);

      // Close recorder window
      closeWindow();
    } catch (err) {
      console.error('Error saving recording:', err);
      error = `Failed to save recording: ${err.message || err}`;
    }
  }

  /**
   * Auto-import recording to media library
   */
  async function autoImportRecording(filePath) {
    try {
      console.log('Auto-importing recording...');

      // Get video metadata
      const metadata = await invoke('pick_video_file_by_path', { path: filePath });
      const clipId = `clip-${Date.now()}`;

      // Add to clips store
      clipsStore.update(clips => [
        ...clips,
        {
          id: clipId,
          filename: metadata.filename,
          path: metadata.path,
          duration: metadata.duration,
          resolution: metadata.resolution
        }
      ]);

      // Generate thumbnail
      try {
        const thumbnailTimestamp = Math.min(1.0, metadata.duration ?? 1.0);
        const thumbnail = await invoke('generate_thumbnail', {
          videoPath: metadata.path,
          timestamp: thumbnailTimestamp
        });

        clipsStore.update(clips => {
          return clips.map(c =>
            c.id === clipId ? { ...c, thumbnail: String(thumbnail) } : c
          );
        });
      } catch (err) {
        console.error('Error generating thumbnail:', err);
      }

      // Generate filmstrip in background
      generateFilmstripForClip(clipId).catch(err => {
        console.error('Error generating filmstrip:', err);
      });

      console.log('Recording auto-imported successfully');
    } catch (err) {
      console.error('Error auto-importing recording:', err);
      error = `Failed to import recording: ${err.message || err}`;
    }
  }

  /**
   * Clean up resources
   */
  function cleanup() {
    console.log('Cleaning up recording resources...');

    // Stop timer
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }

    // Stop animation frame
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId);
      animationFrameId = null;
    }

    // Stop streams
    if (screenStream) {
      screenStream.getTracks().forEach(track => track.stop());
      screenStream = null;
    }
    if (webcamStream) {
      webcamStream.getTracks().forEach(track => track.stop());
      webcamStream = null;
    }

    // Update state
    isRecording = false;
    recordingStore.update(state => ({
      ...state,
      isRecording: false,
      source: null,
      startTime: null,
      recordedDuration: 0
    }));
  }

  /**
   * Close recorder window
   */
  async function closeWindow() {
    try {
      await invoke('close_recorder_window');
    } catch (err) {
      console.error('Error closing window:', err);
    }
  }

  // Cleanup on component destroy
  onDestroy(() => {
    cleanup();
  });
</script>

<div class="flex flex-col h-screen w-full bg-background overflow-hidden">
  <!-- Status Bar -->
  <div class="flex items-center justify-between px-4 py-2 bg-muted border-b shrink-0">
    <div class="flex items-center gap-2">
      {#if isRecording}
        <Circle class="w-3 h-3 fill-destructive text-destructive animate-pulse" />
        <span class="text-sm text-destructive font-medium">Recording...</span>
      {:else}
        <Circle class="w-3 h-3 text-muted-foreground" />
        <span class="text-sm text-muted-foreground">Ready</span>
      {/if}
    </div>
    <Badge variant="secondary" class="font-mono text-xs">
      {Math.floor(recordedDuration / 60)}:{(recordedDuration % 60).toString().padStart(2, '0')}
    </Badge>
  </div>

  <!-- Preview Area (webcam feed during recording) -->
  <div class="flex-1 bg-black flex items-center justify-center overflow-hidden min-h-0">
    {#if isRecording && webcamStream}
      <!-- svelte-ignore a11y_media_has_caption -->
      <video
        bind:this={webcamPreviewElement}
        autoplay
        muted
        class="w-full h-full object-contain"
      ></video>
    {:else}
      <p class="text-sm text-muted-foreground">
        Webcam preview will appear here
      </p>
    {/if}

    <!-- Hidden canvas for compositing -->
    <canvas bind:this={canvas} class="hidden"></canvas>
  </div>

  <!-- Source Selector (locked to "Both" for MVP) -->
  <div class="flex items-center gap-4 px-4 py-2 bg-muted border-t shrink-0">
    <span class="text-xs text-muted-foreground">Mode:</span>
    <Badge variant="outline" class="text-xs">Screen + Webcam</Badge>
  </div>

  <!-- Error Display -->
  {#if error}
    <Alert.Root variant="destructive" class="mx-4 my-2 shrink-0">
      <Alert.Description class="text-xs">{error}</Alert.Description>
    </Alert.Root>
  {/if}

  <!-- Action Buttons -->
  <div class="flex gap-2 p-3 bg-muted border-t shrink-0">
    {#if !isRecording}
      <Button onclick={startRecording} class="flex-1" variant="default">
        <Circle class="w-4 h-4 mr-2" />
        Start Recording
      </Button>
      <Button onclick={closeWindow} variant="outline" class="flex-1">
        <X class="w-4 h-4 mr-2" />
        Close
      </Button>
    {:else}
      <Button onclick={stopRecording} variant="destructive" class="flex-1">
        Stop & Save
      </Button>
      <Button onclick={cancelRecording} variant="outline" class="flex-1">
        Cancel
      </Button>
    {/if}
  </div>
</div>

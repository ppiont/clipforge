<script>
  /**
   * Recorder View
   * Floating recorder window (400x500px)
   * Implements screen + webcam PiP recording with canvas compositing
   */

  import { invoke } from '@tauri-apps/api/core';
  import { onDestroy } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import * as Alert from '$lib/components/ui/alert';
  import * as ToggleGroup from '$lib/components/ui/toggle-group';
  import { recordingStore } from '../stores/recording.js';
  import { clipsStore, generateFilmstripForClip } from '../stores/clips.js';
  import { Circle, X, Monitor, Camera, MonitorPlay } from '@lucide/svelte';

  // Recording state
  let isRecording = $state(false);
  let recordedDuration = $state(0);
  let timerInterval = null;
  let error = $state('');
  let selectedSources = $state(['screen', 'webcam']); // Array of selected sources
  let recordingMode = $state('both'); // Derived: 'screen', 'webcam', or 'both'

  // Derive recording mode from selected sources
  $effect(() => {
    const hasScreen = selectedSources.includes('screen');
    const hasWebcam = selectedSources.includes('webcam');

    if (hasScreen && hasWebcam) {
      recordingMode = 'both';
    } else if (hasScreen) {
      recordingMode = 'screen';
    } else if (hasWebcam) {
      recordingMode = 'webcam';
    } else {
      // Default to both if nothing selected
      selectedSources = ['screen', 'webcam'];
      recordingMode = 'both';
    }
  });

  // Live webcam preview when webcam is toggled (even before recording)
  $effect(() => {
    const webcamEnabled = selectedSources.includes('webcam');

    if (webcamEnabled && !isRecording && !webcamStream) {
      // Start webcam preview
      startWebcamPreview();
    } else if (!webcamEnabled && !isRecording && webcamStream) {
      // Stop webcam preview
      stopWebcamPreview();
    }
  });

  // Attach webcam stream to preview element when available
  $effect(() => {
    if (webcamStream && webcamPreviewElement) {
      webcamPreviewElement.srcObject = webcamStream;
    }
  });

  // Media elements and streams (reactive for UI updates)
  let screenStream = $state(null);
  let webcamStream = $state(null);
  let compositeStream = null;
  let mediaRecorder = null;
  let recordedChunks = [];

  // Canvas references
  let canvas = $state(null);
  let ctx = null;
  let screenVideo = null;
  let webcamVideo = null;
  let webcamPreviewElement = $state(null);

  // Animation frame ID
  let animationFrameId = null;

  /**
   * Start recording: Acquire streams and set up canvas compositing
   */
  async function startRecording() {
    try {
      error = '';
      console.log(`Starting ${recordingMode} recording...`);

      // Step 1: Get screen stream (if needed)
      if (recordingMode === 'screen' || recordingMode === 'both') {
        try {
          // Try to get screen with audio first
          screenStream = await navigator.mediaDevices.getDisplayMedia({
            video: {
              cursor: 'always',
              displaySurface: 'monitor'
            },
            audio: {
              echoCancellation: true,
              noiseSuppression: true,
              sampleRate: 48000
            }
          });

          const hasAudio = screenStream.getAudioTracks().length > 0;
          console.log(`Screen stream acquired ${hasAudio ? 'with' : 'without'} audio`);

          if (!hasAudio) {
            console.warn('Screen audio not available - this is common on macOS. Microphone audio will be used instead.');
          }
        } catch (err) {
          // If audio fails, try video-only
          console.warn('Failed to get screen with audio, trying video-only:', err);
          screenStream = await navigator.mediaDevices.getDisplayMedia({
            video: {
              cursor: 'always',
              displaySurface: 'monitor'
            },
            audio: false
          });
          console.log('Screen stream acquired (video only, no system audio)');
        }
      }

      // Step 2: Get webcam stream (if needed and not already active from preview)
      if (recordingMode === 'webcam' || recordingMode === 'both') {
        // Stop existing preview stream if it exists (doesn't have audio)
        if (webcamStream && recordingMode !== 'screen') {
          webcamStream.getTracks().forEach(track => track.stop());
          webcamStream = null;
        }

        if (!webcamStream) {
          webcamStream = await navigator.mediaDevices.getUserMedia({
            video: {
              width: { ideal: 1280 },
              height: { ideal: 720 }
            },
            audio: {
              echoCancellation: true,
              noiseSuppression: true,
              sampleRate: 48000
            } // Always request audio for webcam/microphone
          });
          console.log('Webcam stream acquired with audio');
        } else {
          console.log('Using existing webcam stream from preview');
        }
      }

      // Step 3: Set up canvas for compositing (if both modes)
      if (recordingMode === 'both') {
        setupCanvas();
      }

      // Step 4: Start MediaRecorder with appropriate stream
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
        source: recordingMode,
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
   * Start MediaRecorder with appropriate stream based on mode
   */
  function startMediaRecorder() {
    let recordStream;

    // Determine which stream to record
    if (recordingMode === 'both') {
      // Capture composite stream from canvas
      recordStream = canvas.captureStream(30);

      // Add audio tracks from both screen and webcam
      const screenAudioTrack = screenStream.getAudioTracks()[0];
      const webcamAudioTrack = webcamStream.getAudioTracks()[0];

      if (screenAudioTrack) {
        console.log('Adding screen audio track');
        recordStream.addTrack(screenAudioTrack);
      } else {
        console.warn('No screen audio track available (this is common on macOS)');
      }

      if (webcamAudioTrack) {
        console.log('Adding webcam microphone audio track');
        recordStream.addTrack(webcamAudioTrack);
      } else {
        console.warn('No webcam audio track available');
      }

      // Note: MediaRecorder will automatically mix multiple audio tracks
    } else if (recordingMode === 'screen') {
      // Record screen stream directly
      recordStream = screenStream;
    } else if (recordingMode === 'webcam') {
      // Record webcam stream directly
      recordStream = webcamStream;
    }

    // Log final stream configuration
    const videoTracks = recordStream.getVideoTracks();
    const audioTracks = recordStream.getAudioTracks();
    console.log(`Recording stream: ${videoTracks.length} video track(s), ${audioTracks.length} audio track(s)`);
    audioTracks.forEach((track, i) => {
      console.log(`  Audio track ${i}: ${track.label} (enabled: ${track.enabled})`);
    });

    // Create MediaRecorder
    recordedChunks = [];
    mediaRecorder = new MediaRecorder(recordStream, {
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
      // Cleanup after save completes
      cleanup();
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
      // Don't cleanup here - let onstop handler do it after save completes
    }
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

      // Generate filename (save as WebM temporarily, will convert to MP4)
      const now = new Date();
      const timestamp = now.toISOString().replace(/[:.]/g, '-').split('T');
      const tempFilename = `recording_${timestamp[0]}_${timestamp[1].slice(0, 8)}_temp.webm`;

      // Save via Tauri command
      console.log(`Saving recording as ${tempFilename}...`);
      const tempFilePath = await invoke('save_recording', {
        blob: Array.from(uint8Array),
        filename: tempFilename
      });
      console.log(`Recording saved to: ${tempFilePath}`);

      // Convert to MP4 for better compatibility
      console.log('Converting to MP4...');
      const mp4Filename = `recording_${timestamp[0]}_${timestamp[1].slice(0, 8)}.mp4`;
      const mp4FilePath = await invoke('convert_webm_to_mp4', {
        inputPath: tempFilePath,
        outputFilename: mp4Filename
      });
      console.log(`Converted to MP4: ${mp4FilePath}`);

      // Auto-import the MP4 to timeline
      await autoImportRecording(mp4FilePath);

      console.log('Recording saved and imported successfully!');

      // Close recorder window after a short delay (so user sees success)
      setTimeout(() => {
        closeWindow();
      }, 1000);
    } catch (err) {
      console.error('Error saving recording:', err);
      error = `Failed to save recording: ${err.message || err}`;
      // Don't close window on error so user can see the error message
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
   * Start webcam preview (before recording starts)
   */
  async function startWebcamPreview() {
    try {
      console.log('Starting webcam preview...');
      webcamStream = await navigator.mediaDevices.getUserMedia({
        video: {
          width: { ideal: 1280 },
          height: { ideal: 720 }
        },
        audio: false
      });

      // Attach to preview element
      if (webcamPreviewElement) {
        webcamPreviewElement.srcObject = webcamStream;
      }

      console.log('Webcam preview started');
    } catch (err) {
      console.error('Error starting webcam preview:', err);
      error = `Failed to access webcam: ${err.message || err}`;
    }
  }

  /**
   * Stop webcam preview (when toggle is disabled)
   */
  function stopWebcamPreview() {
    console.log('Stopping webcam preview...');
    if (webcamStream) {
      webcamStream.getTracks().forEach(track => track.stop());
      webcamStream = null;
    }
    if (webcamPreviewElement) {
      webcamPreviewElement.srcObject = null;
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
  <div class="flex items-center justify-between px-4 py-3 border-b shrink-0 transition-colors duration-200 {isRecording ? 'bg-destructive/10 border-destructive/30' : 'bg-muted'}">
    <div class="flex items-center gap-2">
      {#if isRecording}
        <Circle class="w-4 h-4 fill-destructive text-destructive animate-pulse" />
        <span class="text-sm text-destructive font-semibold">Recording...</span>
      {:else}
        <Circle class="w-4 h-4 text-muted-foreground" />
        <span class="text-sm text-muted-foreground font-medium">Ready</span>
      {/if}
    </div>
    <Badge variant={isRecording ? "outline" : "secondary"} class="font-mono text-xs tabular-nums {isRecording ? 'border-destructive/50 text-destructive' : ''}">
      {Math.floor(recordedDuration / 60)}:{(recordedDuration % 60).toString().padStart(2, '0')}
    </Badge>
  </div>

  <!-- Preview Area -->
  <div class="flex-1 bg-slate-950 flex items-center justify-center overflow-hidden min-h-0">
    {#if webcamStream}
      <!-- Show webcam feed (live preview or during recording) -->
      <!-- svelte-ignore a11y_media_has_caption -->
      <video
        bind:this={webcamPreviewElement}
        autoplay
        muted
        class="w-full h-full object-contain scale-x-[-1]"
      ></video>
    {:else if isRecording && recordingMode === 'screen'}
      <p class="text-sm text-muted-foreground">
        Recording screen...
      </p>
    {:else}
      <div class="text-center space-y-3">
        <div class="flex items-center justify-center gap-2">
          {#if selectedSources.includes('screen')}
            <Monitor class="text-muted-foreground/70" />
          {/if}
          {#if selectedSources.includes('webcam')}
            <Camera class="text-muted-foreground/70" />
          {/if}
        </div>
        <p class="text-sm text-muted-foreground font-medium">
          {recordingMode === 'screen' ? 'Screen recording' :
           recordingMode === 'webcam' ? 'Webcam recording' :
           'Screen + Webcam (PiP)'}
        </p>
        <p class="text-xs text-muted-foreground {selectedSources.includes('webcam') ? 'animate-pulse' : ''}">
          {selectedSources.includes('webcam') ? 'Waiting for webcam...' : 'Select sources below, then click Start'}
        </p>
      </div>
    {/if}

    <!-- Hidden canvas for compositing -->
    <canvas bind:this={canvas} class="hidden"></canvas>
  </div>

  <!-- Mode Selector -->
  <div class="flex flex-col gap-2 px-4 py-3 bg-muted border-t shrink-0">
    <div class="flex items-center justify-between">
      <span class="text-xs font-medium text-muted-foreground">Recording Sources</span>
      {#if recordingMode === 'both'}
        <Badge variant="secondary" class="text-xs">Screen + Webcam</Badge>
      {/if}
    </div>
    <ToggleGroup.Root type="multiple" bind:value={selectedSources} disabled={isRecording} class="justify-start gap-2">
      <ToggleGroup.Item value="screen" aria-label="Screen recording" class="flex-1 transition-all duration-150">
        <Monitor class="mr-2" />
        Screen
      </ToggleGroup.Item>
      <ToggleGroup.Item value="webcam" aria-label="Webcam recording" class="flex-1 transition-all duration-150">
        <Camera class="mr-2" />
        Webcam
      </ToggleGroup.Item>
    </ToggleGroup.Root>
  </div>

  <!-- Error Display -->
  {#if error}
    <Alert.Root variant="destructive" class="mx-4 my-3 shrink-0">
      <Alert.Description class="text-sm">{error}</Alert.Description>
    </Alert.Root>
  {/if}

  <!-- Action Buttons -->
  <div class="flex gap-2 px-4 py-3 bg-muted border-t shrink-0">
    {#if !isRecording}
      <Button onclick={startRecording} class="flex-1 transition-all active:scale-95" variant="default">
        <Circle class="mr-2" />
        Start Recording
      </Button>
      <Button onclick={closeWindow} variant="outline" class="flex-1 transition-all active:scale-95">
        <X class="mr-2" />
        Close
      </Button>
    {:else}
      <Button onclick={stopRecording} variant="destructive" class="flex-1 transition-all active:scale-95">
        Stop & Save
      </Button>
      <Button onclick={cancelRecording} variant="outline" class="flex-1 transition-all active:scale-95">
        Cancel
      </Button>
    {/if}
  </div>
</div>

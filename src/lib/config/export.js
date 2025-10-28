/**
 * Export Configuration
 * Centralized configuration for video export options
 */

/**
 * Available export resolutions
 * @type {Array<{value: string, label: string, description: string, width?: number, height?: number}>}
 */
export const EXPORT_RESOLUTIONS = [
  {
    value: 'Source',
    label: 'Source (Original)',
    description: 'Keep original resolution'
  },
  {
    value: '720p',
    label: '720p HD',
    description: '1280 × 720',
    width: 1280,
    height: 720
  },
  {
    value: '1080p',
    label: '1080p Full HD',
    description: '1920 × 1080',
    width: 1920,
    height: 1080
  },
  {
    value: '1440p',
    label: '1440p 2K',
    description: '2560 × 1440',
    width: 2560,
    height: 1440
  },
  {
    value: '4K',
    label: '4K Ultra HD',
    description: '3840 × 2160',
    width: 3840,
    height: 2160
  }
];

/**
 * Available export formats
 * @type {Array<{value: string, label: string, description: string, icon: string}>}
 */
export const EXPORT_FORMATS = [
  {
    value: 'mp4',
    label: 'MP4',
    description: 'H.264/AAC - Best compatibility',
    icon: 'FileVideo'
  },
  {
    value: 'webm',
    label: 'WebM',
    description: 'VP9/Opus - Modern web format',
    icon: 'Globe'
  },
  {
    value: 'mov',
    label: 'MOV',
    description: 'Apple QuickTime format',
    icon: 'Apple'
  }
];

/**
 * Calculate estimated file size based on resolution and duration
 * Rough heuristic: bitrate varies by resolution
 * @param {string} resolution - Resolution value (e.g., '1080p', 'Source')
 * @param {number} durationSeconds - Total duration in seconds
 * @param {string} format - Export format
 * @returns {string} Formatted file size estimate
 */
export function estimateFileSize(resolution, durationSeconds, format = 'mp4') {
  // Average bitrates in Mbps for different resolutions (H.264)
  const bitrates = {
    '720p': 5,
    '1080p': 8,
    '1440p': 16,
    '4K': 45,
    'Source': 8 // Default to 1080p bitrate
  };

  // Format multipliers (relative to MP4)
  const formatMultipliers = {
    'mp4': 1.0,
    'webm': 0.8, // WebM/VP9 is more efficient
    'mov': 1.2  // MOV can be larger
  };

  const bitrate = bitrates[resolution] || 8;
  const multiplier = formatMultipliers[format] || 1.0;

  // Calculate size in MB: (bitrate in Mbps * duration in seconds * format multiplier) / 8 bits per byte
  const sizeInMB = (bitrate * durationSeconds * multiplier) / 8;

  if (sizeInMB < 1) {
    return `${Math.round(sizeInMB * 1024)} KB`;
  } else if (sizeInMB < 1000) {
    return `${Math.round(sizeInMB)} MB`;
  } else {
    return `${(sizeInMB / 1024).toFixed(1)} GB`;
  }
}

import { writable, get } from 'svelte/store';
import { timelineStore } from './timeline.js';

/**
 * @typedef {Object} UndoState
 * @property {any[]} undoStack - Stack of previous states
 * @property {any[]} redoStack - Stack of undone states
 * @property {number} maxHistory - Maximum history size
 */

const MAX_HISTORY = 50;

/**
 * Undo/Redo Manager for Timeline Operations
 * Implements command pattern for timeline state management
 */
class UndoManager {
  constructor() {
    this.undoStack = [];
    this.redoStack = [];
    this.maxHistory = MAX_HISTORY;
  }

  /**
   * Save current timeline state to undo stack
   */
  saveState() {
    const currentState = get(timelineStore);
    const stateCopy = JSON.stringify({
      clips: currentState.clips,
      duration: currentState.duration,
      playhead: currentState.playhead
    });

    this.undoStack.push(stateCopy);

    // Limit stack size
    if (this.undoStack.length > this.maxHistory) {
      this.undoStack.shift();
    }

    // Clear redo stack when new action is performed
    this.redoStack = [];

    console.log('Saved undo state. Stack size:', this.undoStack.length);
  }

  /**
   * Undo last operation
   * @returns {boolean} - True if undo was successful
   */
  undo() {
    if (this.undoStack.length === 0) {
      console.log('Nothing to undo');
      return false;
    }

    // Save current state to redo stack
    const currentState = get(timelineStore);
    const currentCopy = JSON.stringify({
      clips: currentState.clips,
      duration: currentState.duration,
      playhead: currentState.playhead
    });
    this.redoStack.push(currentCopy);

    // Restore previous state
    const previousState = this.undoStack.pop();
    const restored = JSON.parse(previousState);

    timelineStore.update(state => ({
      ...state,
      ...restored
    }));

    console.log('Undo performed. Undo stack:', this.undoStack.length, 'Redo stack:', this.redoStack.length);
    return true;
  }

  /**
   * Redo last undone operation
   * @returns {boolean} - True if redo was successful
   */
  redo() {
    if (this.redoStack.length === 0) {
      console.log('Nothing to redo');
      return false;
    }

    // Save current state to undo stack
    const currentState = get(timelineStore);
    const currentCopy = JSON.stringify({
      clips: currentState.clips,
      duration: currentState.duration,
      playhead: currentState.playhead
    });
    this.undoStack.push(currentCopy);

    // Restore next state
    const nextState = this.redoStack.pop();
    const restored = JSON.parse(nextState);

    timelineStore.update(state => ({
      ...state,
      ...restored
    }));

    console.log('Redo performed. Undo stack:', this.undoStack.length, 'Redo stack:', this.redoStack.length);
    return true;
  }

  /**
   * Check if undo is available
   * @returns {boolean}
   */
  canUndo() {
    return this.undoStack.length > 0;
  }

  /**
   * Check if redo is available
   * @returns {boolean}
   */
  canRedo() {
    return this.redoStack.length > 0;
  }

  /**
   * Clear all history
   */
  clear() {
    this.undoStack = [];
    this.redoStack = [];
    console.log('Undo/redo history cleared');
  }
}

// Export singleton instance
export const undoManager = new UndoManager();

// Export store for reactive UI updates
export const undoRedoStore = writable({
  canUndo: false,
  canRedo: false
});

// Update store when undo/redo state changes
export function updateUndoRedoState() {
  undoRedoStore.set({
    canUndo: undoManager.canUndo(),
    canRedo: undoManager.canRedo()
  });
}

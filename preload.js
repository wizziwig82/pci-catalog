// Preload script for secure context bridge between main and renderer processes
const { contextBridge, ipcRenderer } = require('electron');
const os = require('os');
const path = require('path');

// Expose protected methods that allow the renderer process to use
// the ipcRenderer without exposing the entire object
contextBridge.exposeInMainWorld('api', {
  // File system operations
  selectFiles: () => ipcRenderer.invoke('open-file-dialog'),
  
  // Settings management
  saveSettings: (settings) => ipcRenderer.invoke('save-settings', settings),
  getSettings: () => ipcRenderer.invoke('get-settings'),
  
  // System information
  platform: () => process.platform,
  tempDir: () => os.tmpdir(),
  homedir: () => os.homedir(),
  
  // Will add more methods here as needed (for MongoDB, R2, etc.)
  
  // Event handling
  on: (channel, callback) => {
    // Whitelist channels that can be listened to
    const validChannels = [
      'file-processed', 
      'upload-progress', 
      'error'
    ];
    if (validChannels.includes(channel)) {
      // Deliberately strip event as it includes `sender` 
      const subscription = (event, ...args) => callback(...args);
      ipcRenderer.on(channel, subscription);
      
      // Return a function to remove this event listener
      return () => {
        ipcRenderer.removeListener(channel, subscription);
      };
    }
  }
});

// You can also expose specific Node.js modules or functionality here
// that you want available in the renderer process

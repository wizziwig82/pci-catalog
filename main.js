// Main process for the Electron application
const { app, BrowserWindow, ipcMain, dialog } = require('electron');
const path = require('path');
const fs = require('fs');
const Store = require('electron-store');

// Initialize config store
const store = new Store();

// Keep a global reference of the window object to avoid garbage collection
let mainWindow;

function createWindow() {
  // Create the browser window
  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      nodeIntegration: false, // Security: disable Node integration in renderer
      contextIsolation: true, // Security: enable context isolation
      preload: path.join(__dirname, 'preload.js') // Use preload script
    },
    icon: path.join(__dirname, 'assets/icons/icon.png')
  });

  // Load the app's main HTML file
  mainWindow.loadFile(path.join(__dirname, 'renderer/index.html'));

  // Open DevTools in development mode
  if (process.argv.includes('--dev')) {
    mainWindow.webContents.openDevTools();
  }

  // Handle window closing
  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

// Create window when Electron has finished initialization
app.whenReady().then(() => {
  createWindow();

  // On macOS, recreate window when dock icon is clicked
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

// Quit when all windows are closed (except on macOS)
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

// IPC handlers for file operations
ipcMain.handle('open-file-dialog', async () => {
  const { canceled, filePaths } = await dialog.showOpenDialog({
    properties: ['openFile', 'multiSelections'],
    filters: [
      { name: 'All Files', extensions: ['*'] },
      { name: 'Documents', extensions: ['pdf', 'doc', 'docx', 'txt', 'rtf', 'csv', 'xls', 'xlsx'] },
      { name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'tiff'] },
      { name: 'Audio', extensions: ['mp3', 'wav', 'flac', 'aac', 'ogg'] },
      { name: 'Video', extensions: ['mp4', 'avi', 'mov', 'wmv', 'mkv'] }
    ]
  });
  
  if (canceled) {
    return [];
  }
  
  return filePaths;
});

// Example of a handler for saving app settings
ipcMain.handle('save-settings', async (event, settings) => {
  store.set('settings', settings);
  return true;
});

ipcMain.handle('get-settings', async () => {
  return store.get('settings') || {};
});

// Initialize any required application components
function initializeApp() {
  // You'll add initialization code for MongoDB, R2, etc. here later
  console.log('PCI File Manager initialized');
}

// Call initialization after app is ready
app.whenReady().then(initializeApp);

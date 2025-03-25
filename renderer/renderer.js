// Renderer process main script
document.addEventListener('DOMContentLoaded', () => {
  // DOM Elements
  const selectFilesBtn = document.getElementById('selectFilesBtn');
  const fileList = document.getElementById('fileList');
  const settingsForm = document.getElementById('settingsForm');
  
  // Load saved settings
  loadSettings();
  
  // Event Listeners
  selectFilesBtn.addEventListener('click', handleFileSelection);
  settingsForm.addEventListener('submit', handleSettingsSave);
  
  // File selection handler
  async function handleFileSelection() {
    const files = await window.api.selectFiles();
    displaySelectedFiles(files);
  }
  
  // Display selected files in the UI
  function displaySelectedFiles(filePaths) {
    fileList.innerHTML = '';
    
    if (filePaths.length === 0) {
      return;
    }
    
    filePaths.forEach(filePath => {
      const fileItem = document.createElement('div');
      fileItem.className = 'file-item';
      
      const fileName = filePath.split(/[\\/]/).pop();
      
      fileItem.innerHTML = `
        <span class="file-name">${fileName}</span>
        <span class="file-path">${filePath}</span>
      `;
      
      fileList.appendChild(fileItem);
    });
  }
  
  // Settings form handlers
  async function loadSettings() {
    const settings = await window.api.getSettings();
    
    if (Object.keys(settings).length > 0) {
      document.getElementById('r2AccountId').value = settings.r2AccountId || '';
      document.getElementById('r2AccessKeyId').value = settings.r2AccessKeyId || '';
      document.getElementById('r2SecretKey').value = settings.r2SecretKey || '';
      document.getElementById('r2BucketName').value = settings.r2BucketName || '';
      document.getElementById('mongoUri').value = settings.mongoUri || '';
    }
  }
  
  async function handleSettingsSave(event) {
    event.preventDefault();
    
    const settings = {
      r2AccountId: document.getElementById('r2AccountId').value,
      r2AccessKeyId: document.getElementById('r2AccessKeyId').value,
      r2SecretKey: document.getElementById('r2SecretKey').value,
      r2BucketName: document.getElementById('r2BucketName').value,
      mongoUri: document.getElementById('mongoUri').value
    };
    
    const success = await window.api.saveSettings(settings);
    
    if (success) {
      alert('Settings saved successfully!');
    } else {
      alert('Failed to save settings.');
    }
  }
  
  // Setup event listeners for events from main process
  const removeProgressListener = window.api.on('upload-progress', (progress) => {
    console.log('Upload progress:', progress);
    // Update progress UI
  });
  
  const removeErrorListener = window.api.on('error', (error) => {
    console.error('Error:', error);
    alert(`Error: ${error}`);
  });
  
  // Clean up event listeners when window is closed
  window.addEventListener('beforeunload', () => {
    removeProgressListener();
    removeErrorListener();
  });
});

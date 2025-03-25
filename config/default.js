/**
 * Default configuration for PCI File Manager
 */

module.exports = {
  app: {
    name: 'PCI File Manager',
    version: '1.0.0',
  },
  storage: {
    // Default settings for Cloudflare R2
    r2: {
      accountId: '',
      accessKeyId: '',
      secretAccessKey: '',
      bucketName: '',
      publicUrl: ''
    }
  },
  database: {
    // Default MongoDB settings
    mongodb: {
      uri: 'mongodb://localhost:27017/pci-files',
      options: {
        useNewUrlParser: true,
        useUnifiedTopology: true
      }
    }
  },
  security: {
    // Security settings
    encryptionKey: '', // Will be generated on first run if empty
    fileAccessControl: 'permissive' // 'permissive', 'moderate', 'strict'
  },
  ui: {
    // UI settings
    theme: 'light', // 'light', 'dark', 'system'
    listViewMode: 'grid', // 'list', 'grid'
    thumbnailSize: 'medium' // 'small', 'medium', 'large'
  }
}; 
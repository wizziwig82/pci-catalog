# Deployment Guide

**Date**: [Current Date]

This document outlines the deployment processes and configuration for the PCI File Manager application, covering both the Electron desktop application and the backend API services.

## Overview

The PCI File Manager consists of two main components that need to be deployed:

1. **Electron Desktop Application**: Distributed to end users as an installable application
2. **Backend API Services**: Hosted on a server to provide data and file management capabilities

## Environment Setup

### Required Services

Before deployment, ensure the following services are set up:

1. **MongoDB Database**: For storing application data
2. **Cloudflare R2 Storage**: For storing audio files and images
3. **Authentication Provider**: JWT issuing service
4. **Redis** (optional): For job queues and caching

### Environment Variables

The application uses environment variables for configuration. These should be set up in each environment:

#### Backend API Environment Variables

```
# Server Configuration
PORT=3000
NODE_ENV=production

# MongoDB Connection
MONGODB_URI=mongodb+srv://username:password@hostname/database

# Cloudflare R2 Configuration
R2_ACCOUNT_ID=your_account_id
R2_ACCESS_KEY_ID=your_access_key
R2_SECRET_ACCESS_KEY=your_secret_key
R2_BUCKET_AUDIO=pci-audio-files
R2_BUCKET_IMAGES=pci-cover-art

# JWT Authentication
JWT_SECRET=your_jwt_secret_key
JWT_EXPIRATION=3600
REFRESH_TOKEN_SECRET=your_refresh_token_secret
REFRESH_TOKEN_EXPIRATION=604800

# Redis Configuration (Optional)
REDIS_URL=redis://username:password@hostname:port

# Logging
LOG_LEVEL=info

# CORS Configuration
ALLOWED_ORIGINS=https://app.example.com,https://admin.example.com
```

#### Electron App Environment Variables

For the Electron app, environment variables are compiled into the application at build time and stored in a `.env` file:

```
# API Configuration
REACT_APP_API_URL=https://api.example.com

# Feature Flags
REACT_APP_ENABLE_ANALYTICS=true
REACT_APP_ENABLE_EXPERIMENTAL_FEATURES=false

# Version Info
REACT_APP_VERSION=$npm_package_version
```

## Backend API Deployment

### Prerequisites

- Node.js v16.x or higher
- npm or yarn
- MongoDB database instance
- Cloudflare R2 account and credentials
- SSL certificate for HTTPS

### Production Deployment

#### Option 1: Docker Deployment

1. **Build the Docker image**:

```bash
docker build -t pci-file-manager-api .
```

2. **Run the container**:

```bash
docker run -d \
  --name pci-api \
  -p 3000:3000 \
  --env-file .env.production \
  pci-file-manager-api
```

3. **Docker Compose setup**:

```yaml
# docker-compose.yml
version: '3'

services:
  api:
    build: .
    ports:
      - "3000:3000"
    env_file:
      - .env.production
    restart: always
    depends_on:
      - redis
  
  redis:
    image: redis:alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    restart: always

volumes:
  redis-data:
```

Run with:

```bash
docker-compose up -d
```

#### Option 2: Traditional Node.js Deployment

1. **Install dependencies**:

```bash
npm ci --production
```

2. **Build the application**:

```bash
npm run build
```

3. **Start the server using PM2**:

```bash
pm2 start dist/server.js --name pci-api
```

4. **Setup PM2 to start on server reboot**:

```bash
pm2 startup
pm2 save
```

### Scaling Considerations

For high-load scenarios, consider:

1. **Load Balancing**: Deploy multiple API instances behind a load balancer
2. **Database Scaling**: Use MongoDB Atlas or a similar solution for database scaling
3. **Horizontal Scaling**: Deploy API services across multiple servers
4. **CDN Integration**: Use a CDN for static assets

### Monitoring and Logging

1. **PM2 Monitoring**:

```bash
pm2 monit
```

2. **Logging to external services**:
   - Configure Winston logger to send logs to a service like Papertrail, Loggly, or ELK stack
   - Use Sentry or similar for error tracking

3. **Health Checks**:
   - Implement a `/health` endpoint for monitoring services
   - Set up uptime monitoring with a service like UptimeRobot

## Electron Application Deployment

### Prerequisites

- Node.js v16.x or higher
- npm or yarn
- Electron-builder configured in package.json

### Building for Distribution

1. **Install dependencies**:

```bash
npm ci
```

2. **Build the application**:

```bash
# For all platforms
npm run build

# For specific platforms
npm run build:mac
npm run build:win
npm run build:linux
```

This will generate installers in the `dist` directory.

### Code Signing

#### macOS Code Signing

1. **Obtain an Apple Developer Certificate**:
   - Register for an Apple Developer account
   - Create a Developer ID Application certificate in Apple's Developer Portal
   - Download and install the certificate in your keychain

2. **Configure code signing in package.json**:

```json
"build": {
  "mac": {
    "category": "public.app-category.music",
    "hardenedRuntime": true,
    "gatekeeperAssess": false,
    "entitlements": "build/entitlements.mac.plist",
    "entitlementsInherit": "build/entitlements.mac.plist",
    "identity": "Developer ID Application: Your Company Name (ABCDEF1234)"
  }
}
```

3. **Create entitlements file**:

```xml
<!-- build/entitlements.mac.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>
    <key>com.apple.security.cs.allow-dyld-environment-variables</key>
    <true/>
    <key>com.apple.security.automation.apple-events</key>
    <true/>
    <key>com.apple.security.device.audio-input</key>
    <true/>
  </dict>
</plist>
```

4. **Notarize the application** (macOS only):

```bash
npx electron-notarize --bundle-id "com.example.pcifilemanager" --apple-id "your.email@example.com" --apple-id-password "@keychain:AC_PASSWORD" --team-id "ABCDEF1234" ./dist/mac/PCI File Manager.app
```

Add this to the `afterSign` hook in electron-builder.

#### Windows Code Signing

1. **Obtain a Code Signing Certificate**:
   - Purchase a code signing certificate from a trusted Certificate Authority (CA)
   - Export it as a PFX file with password

2. **Configure code signing in package.json**:

```json
"build": {
  "win": {
    "certificateFile": "path/to/certificate.pfx",
    "certificatePassword": "your-password",
    "verifyUpdateCodeSignature": true
  }
}
```

Alternatively, use environment variables:

```
CSC_LINK=base64-encoded-pfx-file-content
CSC_KEY_PASSWORD=your-password
```

### Auto-Update Configuration

1. **Setup auto-update server**:
   - Store update files on a static hosting service
   - Configure server to serve the correct content type for `.yml` files

2. **Configure update URL in package.json**:

```json
"build": {
  "publish": [
    {
      "provider": "generic",
      "url": "https://updates.example.com/pci-file-manager"
    }
  ]
}
```

3. **Code for handling updates in the app**:

```javascript
// In main process
const { autoUpdater } = require('electron-updater');

autoUpdater.logger = require('electron-log');
autoUpdater.logger.transports.file.level = 'info';

// Check for updates
autoUpdater.checkForUpdatesAndNotify();

// Listen for update events
autoUpdater.on('update-available', () => {
  // Notify user of available update
});

autoUpdater.on('update-downloaded', () => {
  // Prompt user to install update
});
```

## Continuous Integration and Deployment

### GitHub Actions Workflow

```yaml
# .github/workflows/release.yml
name: Build and Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [macos-latest, windows-latest, ubuntu-latest]
    
    steps:
    - uses: actions/checkout@v2
    
    - name: Setup Node.js
      uses: actions/setup-node@v2
      with:
        node-version: '16'
    
    - name: Install Dependencies
      run: npm ci
    
    - name: Build Electron App
      run: npm run build
      env:
        GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        CSC_LINK: ${{ secrets.CSC_LINK }}
        CSC_KEY_PASSWORD: ${{ secrets.CSC_KEY_PASSWORD }}
        APPLE_ID: ${{ secrets.APPLE_ID }}
        APPLE_ID_PASSWORD: ${{ secrets.APPLE_ID_PASSWORD }}
        APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
    
    - name: Upload Artifacts
      uses: actions/upload-artifact@v2
      with:
        name: ${{ matrix.os }}-build
        path: dist
    
    - name: Create GitHub Release
      if: startsWith(github.ref, 'refs/tags/v')
      uses: softprops/action-gh-release@v1
      with:
        files: |
          dist/*.exe
          dist/*.dmg
          dist/*.AppImage
          dist/*.deb
          dist/*.rpm
          dist/*.yml
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Versioning

Follow semantic versioning (`MAJOR.MINOR.PATCH`) for the application. Update the version in `package.json`:

```json
{
  "version": "1.0.0"
}
```

To automate version increments, use:

```bash
# Patch update (1.0.0 -> 1.0.1)
npm version patch

# Minor update (1.0.0 -> 1.1.0)
npm version minor

# Major update (1.0.0 -> 2.0.0)
npm version major
```

## Database Migration

### MongoDB Migration

1. **Create migration scripts**:

```javascript
// migrations/01-add-track-index.js
module.exports = {
  async up(db) {
    await db.collection('tracks').createIndex({ title: 'text', artist: 'text' });
  },

  async down(db) {
    await db.collection('tracks').dropIndex('title_text_artist_text');
  }
};
```

2. **Run migrations using migrate-mongo**:

```bash
# Create a new migration file
npx migrate-mongo create add-new-field-to-users

# Apply pending migrations
npx migrate-mongo up

# Revert last migration
npx migrate-mongo down
```

## Backup and Disaster Recovery

### MongoDB Backup

1. **Regular backups using mongodump**:

```bash
mongodump --uri="mongodb+srv://username:password@hostname/database" --out=/backup/mongo/$(date +"%Y-%m-%d")
```

2. **Set up automated backup job**:

```bash
# Add to crontab
0 1 * * * /path/to/backup-script.sh
```

### R2 Storage Backup

1. **Configure replication between R2 buckets**:
   - Set up a secondary bucket for backups
   - Use Cloudflare Workers to automate replication

2. **Regular metadata backups**:
   - Export file metadata and paths regularly
   - Store this data alongside database backups

## Security Considerations

### API Security Checklist

1. **HTTPS**: Ensure all traffic is encrypted with HTTPS
2. **Authentication**: Implement proper JWT validation and expiration
3. **Rate Limiting**: Add rate limiting to prevent abuse
4. **Input Validation**: Validate all API inputs
5. **CORS Headers**: Configure proper CORS headers
6. **Security Headers**: Set appropriate security headers:
   - Content-Security-Policy
   - X-Content-Type-Options
   - X-Frame-Options
   - X-XSS-Protection

### Electron App Security

1. **Context Isolation**: Enable contextIsolation
2. **Content Security Policy**: Implement a strict CSP
3. **Node Integration**: Disable nodeIntegration in renderers
4. **Permissions**: Use proper permission requests
5. **Regular Updates**: Keep dependencies updated

Example preload script with secure configuration:

```javascript
// preload.js
const { contextBridge, ipcRenderer } = require('electron');

// Expose limited API
contextBridge.exposeInMainWorld('electron', {
  send: (channel, data) => {
    // Whitelist channels
    const validChannels = ['track:process', 'file:upload'];
    if (validChannels.includes(channel)) {
      ipcRenderer.send(channel, data);
    }
  },
  receive: (channel, func) => {
    const validChannels = ['track:progress', 'file:complete'];
    if (validChannels.includes(channel)) {
      ipcRenderer.on(channel, (event, ...args) => func(...args));
    }
  }
});
```

## Testing Before Deployment

Before each production deployment, complete the following tests:

1. **API Tests**:
   - Run automated API tests against a staging environment
   - Test all major API endpoints manually
   - Verify authentication and permissions

2. **Electron App Tests**:
   - Test installation on all target platforms
   - Verify auto-update functionality
   - Test file upload and download
   - Check all major user flows

3. **Performance Tests**:
   - Test system under expected load
   - Verify response times remain under acceptable thresholds

## Rollback Procedures

### API Rollback

1. **Docker rollback**:

```bash
# Get previous image
docker pull pci-file-manager-api:previous-tag

# Stop current container
docker stop pci-api

# Start container with previous image
docker run -d --name pci-api -p 3000:3000 --env-file .env.production pci-file-manager-api:previous-tag
```

2. **Traditional deployment rollback**:

```bash
# Using PM2
pm2 list
pm2 revert 0  # Revert to previous deployment
```

### Electron App Rollback

1. **Disable auto-update for problematic version**:
   - Modify the update server configuration to skip the problematic version
   - Push a new update that fixes the issues

2. **Communicate with users**:
   - Provide download links to previous stable version
   - Document manual downgrade process

## Post-Deployment Verification

After deployment, verify:

1. **API Health**:
   - Check the health endpoint
   - Verify logs for any unusual errors
   - Monitor performance metrics

2. **Electron App**:
   - Test fresh installation
   - Verify update from previous version
   - Check critical features

## Related Documents

- [API Endpoints Reference](../api/endpoints-reference.md)
- [R2 Storage Design](../architecture/r2-storage-design.md)
- [Testing Strategy](testing-strategy.md) 
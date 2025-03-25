# PCI File Manager

A modern file management application built with Electron, integrating with Cloudflare R2 for storage and MongoDB for metadata.

## Features

- Upload and manage files of various types
- Store files in Cloudflare R2 cloud storage
- Track file metadata in MongoDB
- Cross-platform desktop application (Windows, macOS, Linux)
- Modern user interface
- Secure file handling

## Getting Started

### Prerequisites

- Node.js 14.x or later
- npm 6.x or later
- MongoDB (local or cloud instance)
- Cloudflare R2 account

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/pci-file-manager.git
   cd pci-file-manager
   ```

2. Install dependencies:
   ```
   npm install
   ```

3. Start the application in development mode:
   ```
   npm run dev
   ```

## Configuration

You can configure the application through the Settings panel or by directly editing the configuration files in the `config` directory.

### R2 Storage Configuration

You'll need to provide your Cloudflare R2 credentials:
- Account ID
- Access Key ID
- Secret Access Key
- Bucket Name

### MongoDB Configuration

Configure your MongoDB connection in the settings panel or config files.

## Building for Production

To build the application for production:

```
npm run build
```

This will create distributions for your current platform in the `dist` directory.

## Development

### Project Structure

- `main.js` - Electron main process
- `preload.js` - Preload script for secure context bridge
- `renderer/` - Frontend code (HTML, CSS, JS)
- `src/` - Application logic
  - `models/` - Data models
  - `services/` - Core services (DB, Storage)
  - `utils/` - Utility functions
- `config/` - Configuration files
- `assets/` - Application assets
- `docs/` - Documentation

## Documentation

Comprehensive documentation is available in the `docs` directory:

### Architecture
- [Application Requirements](docs/architecture/requirements.md)
- [Architecture Overview](docs/architecture/overview.md)
- [Database Schema](docs/architecture/database-schema.md)
- [R2 Storage Design](docs/architecture/r2-storage-design.md)
- [Authentication Flow](docs/architecture/authentication-flow.md)

### API Reference
- [API Endpoints Reference](docs/api/endpoints-reference.md)

### Guides
- [Development Environment Setup](docs/guides/development-environment-setup.md)
- [Cloudflare R2 Integration](docs/guides/cloudflare-r2-integration-guide.md)
- [Performance Optimization](docs/guides/performance-optimization-guide.md)
- [Localization](docs/guides/localization-guide.md)
- [User Management](docs/guides/user-management-guide.md)
- [Security Best Practices](docs/guides/security-best-practices-guide.md)
- [UI Component Guide](docs/guides/ui-component-guide.md)
- [Testing Strategy](docs/guides/testing-strategy.md)
- [Error Handling](docs/guides/error-handling-guide.md)
- [Deployment](docs/guides/deployment-guide.md)
- [Development Roadmap](docs/guides/development-roadmap.md)

For a complete list of documentation, see the [Documentation Index](docs/README.md).

## License

This project is licensed under the MIT License - see the LICENSE file for details. 
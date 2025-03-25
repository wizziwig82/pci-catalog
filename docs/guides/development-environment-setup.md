# Development Environment Setup Guide

*Last Updated: March 25, 2024*

This guide provides comprehensive instructions for setting up your development environment for contributing to the PCI File Manager application.

## Prerequisites

Before you begin, ensure you have the following installed:

- **Node.js** (v14.x or later) - [Download](https://nodejs.org/)
- **npm** (v6.x or later, included with Node.js)
- **Git** - [Download](https://git-scm.com/downloads)
- **MongoDB** (local or cloud instance)
- **Code editor** (VS Code recommended)
- **Cloudflare R2 account**

## Initial Setup

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/pci-file-manager.git
cd pci-file-manager
```

### 2. Install Dependencies

```bash
npm install
```

### 3. Configure Environment Variables

Create a `.env` file in the root directory:

```
# MongoDB
MONGODB_URI=mongodb://localhost:27017/pci-file-manager
# or your MongoDB Atlas connection string

# Cloudflare R2
CF_ACCOUNT_ID=your-account-id
CF_ACCESS_KEY_ID=your-access-key
CF_SECRET_ACCESS_KEY=your-secret-key
CF_BUCKET_NAME=your-bucket-name

# Application
PORT=3000
NODE_ENV=development
```

## Running the Application

### Development Mode

Start the application in development mode:

```bash
npm run dev
```

This will:
- Start the Electron application
- Enable hot-reloading for frontend changes
- Connect to your configured MongoDB and R2 storage

### Building for Production

To build the application for production:

```bash
npm run build
```

This creates distributions for your current platform in the `dist` directory.

## Development Workflow

### Code Structure

- **main.js** - Electron main process
- **preload.js** - Preload script for secure context bridge
- **renderer/** - Frontend code (HTML, CSS, JS)
- **src/** - Application logic
  - **models/** - Data models
  - **services/** - Core services (DB, Storage)
  - **utils/** - Utility functions
- **config/** - Configuration files
- **assets/** - Application assets

### Pull Request Process

1. Create a feature branch: `git checkout -b feature/your-feature-name`
2. Make your changes
3. Run tests: `npm test`
4. Commit changes: `git commit -m "Description of changes"`
5. Push to GitHub: `git push origin feature/your-feature-name`
6. Create a Pull Request on GitHub

## Debugging

### Electron Dev Tools

The application has Chrome DevTools enabled in development mode. Access them by:

1. Right-click in the application window
2. Select "Inspect Element"

### Common Issues

#### MongoDB Connection Issues

- Verify MongoDB is running: `mongod --version`
- Check connection string in your `.env` file
- Ensure network connectivity to MongoDB Atlas (if using cloud)

#### Electron Startup Problems

- Clear node_modules and reinstall: `rm -rf node_modules && npm install`
- Verify Electron version compatibility with your OS

## Related Documentation

- [Architecture Overview](../architecture/overview.md)
- [Database Schema](../architecture/database-schema.md)
- [API Endpoints Reference](../api/endpoints-reference.md)
- [Testing Strategy](testing-strategy.md)

## Getting Help

If you encounter issues not covered in this guide:

1. Check existing GitHub issues
2. Consult the [Error Handling Guide](error-handling-guide.md)
3. Reach out to the development team 
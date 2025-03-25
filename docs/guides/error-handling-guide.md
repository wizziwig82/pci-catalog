# Error Handling Guide

**Date**: [Current Date]

This document outlines the error handling strategies and patterns for the PCI File Manager application, providing a consistent approach to error management across the codebase.

## Error Handling Principles

The application follows these core principles for error handling:

1. **User-Friendly**: Error messages should be clear and actionable for users
2. **Debuggable**: Errors should provide sufficient technical details for developers
3. **Recoverable**: The application should recover gracefully from errors when possible
4. **Consistent**: Error handling patterns should be consistent throughout the application
5. **Informative**: Errors should be logged with appropriate context

## Error Types

The application categorizes errors into the following types:

### 1. Validation Errors

Errors that occur when user input fails validation rules.

```javascript
// Example Validation Error
class ValidationError extends Error {
  constructor(message, fieldErrors = {}) {
    super(message);
    this.name = 'ValidationError';
    this.code = 'VALIDATION_ERROR';
    this.status = 400;
    this.fieldErrors = fieldErrors;
  }
}

// Usage
throw new ValidationError('Invalid input data', {
  title: 'Title is required',
  artist: 'Artist name must be at least 2 characters'
});
```

### 2. Authentication Errors

Errors related to user authentication.

```javascript
// Example Authentication Error
class AuthenticationError extends Error {
  constructor(message) {
    super(message || 'Authentication failed');
    this.name = 'AuthenticationError';
    this.code = 'AUTHENTICATION_ERROR';
    this.status = 401;
  }
}

// Usage
throw new AuthenticationError('Invalid credentials');
```

### 3. Authorization Errors

Errors related to user permissions.

```javascript
// Example Authorization Error
class AuthorizationError extends Error {
  constructor(message, requiredPermission) {
    super(message || 'Not authorized to perform this action');
    this.name = 'AuthorizationError';
    this.code = 'AUTHORIZATION_ERROR';
    this.status = 403;
    this.requiredPermission = requiredPermission;
  }
}

// Usage
throw new AuthorizationError('You do not have permission to delete tracks', 'canDelete');
```

### 4. Resource Errors

Errors related to resource availability.

```javascript
// Example Resource Error
class ResourceNotFoundError extends Error {
  constructor(resourceType, resourceId) {
    super(`${resourceType} with ID ${resourceId} not found`);
    this.name = 'ResourceNotFoundError';
    this.code = 'RESOURCE_NOT_FOUND';
    this.status = 404;
    this.resourceType = resourceType;
    this.resourceId = resourceId;
  }
}

// Usage
throw new ResourceNotFoundError('Track', '60a2b3c4d5e6f7g8h9i0j1k2');
```

### 5. External Service Errors

Errors related to external service dependencies.

```javascript
// Example External Service Error
class ExternalServiceError extends Error {
  constructor(serviceName, message, originalError = null) {
    super(`${serviceName} error: ${message}`);
    this.name = 'ExternalServiceError';
    this.code = 'EXTERNAL_SERVICE_ERROR';
    this.status = 503;
    this.serviceName = serviceName;
    this.originalError = originalError;
  }
}

// Usage
try {
  await r2Client.uploadFile(file);
} catch (error) {
  throw new ExternalServiceError('Cloudflare R2', 'Failed to upload file', error);
}
```

### 6. Operational Errors

Errors related to system operations that might be recoverable.

```javascript
// Example Operational Error
class OperationalError extends Error {
  constructor(operation, message, originalError = null) {
    super(`${operation} failed: ${message}`);
    this.name = 'OperationalError';
    this.code = 'OPERATIONAL_ERROR';
    this.status = 500;
    this.operation = operation;
    this.originalError = originalError;
  }
}

// Usage
try {
  await transcodeFile(file);
} catch (error) {
  throw new OperationalError('Audio transcoding', 'Failed to transcode file', error);
}
```

## Frontend Error Handling

### React Component Error Handling

For component-level error handling, use Error Boundaries:

```jsx
// Example Error Boundary Component
class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error) {
    return { hasError: true, error };
  }

  componentDidCatch(error, errorInfo) {
    // Log the error
    console.error('Component error:', error);
    console.error('Component stack:', errorInfo.componentStack);
    
    // Send to error reporting service
    errorReportingService.captureError(error, {
      componentStack: errorInfo.componentStack,
      ...this.props.errorMetadata
    });
  }

  render() {
    if (this.state.hasError) {
      return this.props.fallback ? (
        this.props.fallback(this.state.error)
      ) : (
        <div className="error-container">
          <h2>Something went wrong.</h2>
          <p>Please try again later or contact support if the problem persists.</p>
          {process.env.NODE_ENV !== 'production' && (
            <details>
              <summary>Error details</summary>
              <pre>{this.state.error.toString()}</pre>
            </details>
          )}
          <button onClick={() => this.setState({ hasError: false, error: null })}>
            Try Again
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}

// Usage
<ErrorBoundary errorMetadata={{ section: 'TrackList' }}>
  <TrackList />
</ErrorBoundary>
```

### API Request Error Handling

For handling API request errors:

```jsx
// Example API request with error handling
const TrackList = () => {
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState(null);
  const [tracks, setTracks] = useState([]);

  useEffect(() => {
    const fetchTracks = async () => {
      try {
        setIsLoading(true);
        setError(null);
        const response = await api.getTracks();
        setTracks(response.tracks);
      } catch (error) {
        setError(handleApiError(error));
      } finally {
        setIsLoading(false);
      }
    };

    fetchTracks();
  }, []);

  if (isLoading) return <LoadingSpinner />;
  
  if (error) {
    return (
      <ErrorDisplay 
        error={error} 
        onRetry={() => {
          setIsLoading(true);
          setError(null);
          fetchTracks();
        }} 
      />
    );
  }

  return (
    <div className="track-list">
      {tracks.map(track => (
        <TrackItem key={track.id} track={track} />
      ))}
    </div>
  );
};

// Helper function to process API errors
const handleApiError = (error) => {
  if (error.response) {
    // Server responded with an error status
    const { status, data } = error.response;
    
    switch (status) {
      case 400:
        return {
          type: 'validation',
          message: 'Please check your input and try again',
          details: data.fieldErrors || {},
          originalError: error
        };
      case 401:
        // Trigger authentication flow
        authService.handleUnauthorized();
        return {
          type: 'authentication',
          message: 'Please log in to continue',
          originalError: error
        };
      case 403:
        return {
          type: 'authorization',
          message: 'You do not have permission to perform this action',
          requiredPermission: data.requiredPermission,
          originalError: error
        };
      case 404:
        return {
          type: 'notFound',
          message: `The requested ${data.resourceType || 'resource'} was not found`,
          originalError: error
        };
      case 429:
        return {
          type: 'rateLimit',
          message: 'Too many requests. Please try again later.',
          originalError: error
        };
      default:
        return {
          type: 'server',
          message: data.message || 'An unexpected error occurred. Please try again later.',
          originalError: error
        };
    }
  } else if (error.request) {
    // Request made but no response received
    return {
      type: 'network',
      message: 'Network error. Please check your connection and try again.',
      originalError: error
    };
  } else {
    // Error setting up request
    return {
      type: 'client',
      message: 'An unexpected client error occurred. Please try again.',
      originalError: error
    };
  }
};
```

### Error Display Components

Create reusable error display components:

```jsx
// Example Error Display Component
const ErrorDisplay = ({ error, onRetry }) => {
  const renderErrorContent = () => {
    switch (error.type) {
      case 'validation':
        return (
          <div>
            <p className="error-message">{error.message}</p>
            {Object.entries(error.details).length > 0 && (
              <ul className="field-errors">
                {Object.entries(error.details).map(([field, message]) => (
                  <li key={field}>
                    <strong>{field}:</strong> {message}
                  </li>
                ))}
              </ul>
            )}
          </div>
        );
      
      case 'authentication':
        return (
          <div>
            <p className="error-message">{error.message}</p>
            <button onClick={() => authService.showLoginModal()}>
              Log In
            </button>
          </div>
        );
      
      case 'network':
        return (
          <div>
            <p className="error-message">{error.message}</p>
            <p>If the problem persists, please contact support.</p>
          </div>
        );
        
      case 'notFound':
        return (
          <div>
            <p className="error-message">{error.message}</p>
            <Link to="/dashboard">Return to Dashboard</Link>
          </div>
        );
      
      default:
        return (
          <p className="error-message">{error.message}</p>
        );
    }
  };

  return (
    <div className="error-container">
      <div className="error-icon">
        <AlertCircleIcon size={24} />
      </div>
      <div className="error-content">
        {renderErrorContent()}
      </div>
      {onRetry && (
        <button className="retry-button" onClick={onRetry}>
          Try Again
        </button>
      )}
      {process.env.NODE_ENV !== 'production' && error.originalError && (
        <details className="error-details">
          <summary>Technical details</summary>
          <pre>{JSON.stringify(error.originalError, null, 2)}</pre>
        </details>
      )}
    </div>
  );
};
```

## Backend Error Handling

### Express Middleware

Use middleware for consistent error handling in Express:

```javascript
// Example error handling middleware
const errorHandler = (err, req, res, next) => {
  // Log the error
  console.error('API Error:', err);
  
  // Track error in monitoring
  if (process.env.NODE_ENV === 'production') {
    errorReportingService.captureError(err, {
      url: req.url,
      method: req.method,
      userId: req.user?.id,
    });
  }
  
  // Handle known error types
  if (err.status && err.code) {
    return res.status(err.status).json({
      error: true,
      code: err.code,
      message: err.message,
      ...(err.fieldErrors && { details: err.fieldErrors }),
      ...(err.requiredPermission && { requiredPermission: err.requiredPermission }),
      ...(process.env.NODE_ENV !== 'production' && { stack: err.stack })
    });
  }
  
  // Handle Mongoose validation errors
  if (err.name === 'ValidationError' && err.errors) {
    const fieldErrors = {};
    
    for (const [field, error] of Object.entries(err.errors)) {
      fieldErrors[field] = error.message;
    }
    
    return res.status(400).json({
      error: true,
      code: 'VALIDATION_ERROR',
      message: 'Validation failed',
      details: fieldErrors,
      ...(process.env.NODE_ENV !== 'production' && { stack: err.stack })
    });
  }
  
  // Handle MongoDB duplicate key errors
  if (err.name === 'MongoError' && err.code === 11000) {
    const field = Object.keys(err.keyValue)[0];
    const value = err.keyValue[field];
    
    return res.status(400).json({
      error: true,
      code: 'DUPLICATE_ERROR',
      message: `The ${field} "${value}" is already in use`,
      details: {
        [field]: `The ${field} "${value}" is already in use`
      },
      ...(process.env.NODE_ENV !== 'production' && { stack: err.stack })
    });
  }
  
  // Default to 500 server error
  return res.status(500).json({
    error: true,
    code: 'INTERNAL_SERVER_ERROR',
    message: 'An unexpected error occurred',
    ...(process.env.NODE_ENV !== 'production' && { stack: err.stack })
  });
};

// Usage
app.use(errorHandler);
```

### Async Route Handlers

Wrap async route handlers to handle promise rejections:

```javascript
// Helper to wrap async route handlers
const asyncHandler = (fn) => (req, res, next) => {
  Promise.resolve(fn(req, res, next)).catch(next);
};

// Usage
app.get('/api/tracks', asyncHandler(async (req, res) => {
  const tracks = await trackService.getTracks(req.query);
  res.json({ tracks });
}));
```

## Electron Process Error Handling

### Main Process Errors

Handle errors in the Electron main process:

```javascript
// Example main process error handling
const { app, dialog } = require('electron');
const log = require('electron-log');

// Configure logging
log.transports.file.level = 'error';
log.transports.console.level = 'debug';

// Handle uncaught exceptions
process.on('uncaughtException', (error) => {
  log.error('Uncaught Exception:', error);
  
  // Show error dialog to user
  dialog.showErrorBox(
    'Application Error',
    'An unexpected error occurred. The application will now restart.'
  );
  
  // Report to error tracking service
  errorReportingService.captureError(error, {
    process: 'main',
    version: app.getVersion()
  });
  
  // Restart the app
  app.relaunch();
  app.exit(1);
});

// Handle unhandled promise rejections
process.on('unhandledRejection', (reason, promise) => {
  log.error('Unhandled Rejection at:', promise, 'reason:', reason);
  
  // Report to error tracking service
  errorReportingService.captureError(reason, {
    process: 'main',
    version: app.getVersion()
  });
});
```

### IPC Error Handling

Handle errors in IPC communication:

```javascript
// Example IPC error handling in main process
const { ipcMain } = require('electron');

ipcMain.handle('track:process', async (event, trackId) => {
  try {
    const result = await processTrack(trackId);
    return { success: true, data: result };
  } catch (error) {
    log.error('Track processing error:', error);
    
    // Return structured error to renderer
    return {
      success: false,
      error: {
        message: error.message,
        code: error.code || 'PROCESSING_ERROR'
      }
    };
  }
});

// Example IPC error handling in renderer process
const processTrack = async (trackId) => {
  const result = await window.electron.invoke('track:process', trackId);
  
  if (!result.success) {
    throw new Error(result.error.message);
  }
  
  return result.data;
};
```

## File Operation Error Handling

Handle errors in file operations:

```javascript
// Example file operation error handling
const processAudioFile = async (filePath) => {
  try {
    // Verify file exists
    try {
      await fs.promises.access(filePath, fs.constants.R_OK);
    } catch (error) {
      throw new OperationalError(
        'File access',
        `Cannot access file at ${filePath}`,
        error
      );
    }
    
    // Check file size
    try {
      const stats = await fs.promises.stat(filePath);
      const fileSizeMB = stats.size / (1024 * 1024);
      
      if (fileSizeMB > MAX_FILE_SIZE_MB) {
        throw new ValidationError(`File size exceeds maximum allowed (${MAX_FILE_SIZE_MB}MB)`);
      }
    } catch (error) {
      if (error instanceof ValidationError) {
        throw error;
      }
      
      throw new OperationalError(
        'File size check',
        `Failed to check file size for ${filePath}`,
        error
      );
    }
    
    // Process file content
    try {
      const data = await fs.promises.readFile(filePath);
      return processAudioData(data);
    } catch (error) {
      throw new OperationalError(
        'File read',
        `Failed to read file data from ${filePath}`,
        error
      );
    }
  } finally {
    // Clean up temporary files
    try {
      await cleanupTempFiles(filePath);
    } catch (cleanupError) {
      console.error('Failed to clean up temporary files:', cleanupError);
      // Don't throw, as this is cleanup code
    }
  }
};
```

## Error Logging and Monitoring

### Logging Strategy

Configure comprehensive error logging:

```javascript
// Example logger configuration
const winston = require('winston');
require('winston-daily-rotate-file');

const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.json()
  ),
  defaultMeta: { service: 'pci-file-manager' },
  transports: [
    // Console transport for development
    new winston.transports.Console({
      format: winston.format.combine(
        winston.format.colorize(),
        winston.format.simple()
      ),
      silent: process.env.NODE_ENV === 'test'
    }),
    
    // File transport for persistent logs
    new winston.transports.DailyRotateFile({
      filename: 'logs/application-%DATE%.log',
      datePattern: 'YYYY-MM-DD',
      maxSize: '20m',
      maxFiles: '14d',
      level: 'info'
    }),
    
    // Separate file for errors
    new winston.transports.DailyRotateFile({
      filename: 'logs/error-%DATE%.log',
      datePattern: 'YYYY-MM-DD',
      maxSize: '20m',
      maxFiles: '30d',
      level: 'error'
    })
  ]
});

// Example error logging
try {
  // Some operation
} catch (error) {
  logger.error('Failed to process track', {
    trackId: track.id,
    userId: user.id,
    errorMessage: error.message,
    errorStack: error.stack,
    errorCode: error.code
  });
  
  throw error;
}
```

### Error Monitoring Service Integration

Integrate with error monitoring services:

```javascript
// Example Sentry integration
const Sentry = require('@sentry/node');
const { CaptureConsole } = require('@sentry/integrations');

Sentry.init({
  dsn: process.env.SENTRY_DSN,
  environment: process.env.NODE_ENV,
  release: `pci-file-manager@${process.env.npm_package_version}`,
  integrations: [
    new CaptureConsole({
      levels: ['error']
    })
  ],
  beforeSend(event, hint) {
    // Sanitize sensitive data
    if (event.request && event.request.headers) {
      delete event.request.headers.authorization;
      delete event.request.headers.cookie;
    }
    
    // Don't send errors in development unless explicitly configured
    if (process.env.NODE_ENV === 'development' && !process.env.SENTRY_REPORT_DEV) {
      return null;
    }
    
    return event;
  }
});

// Usage
try {
  // Some operation
} catch (error) {
  Sentry.captureException(error, {
    tags: {
      component: 'audio-processor'
    },
    extra: {
      trackId: track.id,
      operationType: 'transcode'
    }
  });
  
  throw error;
}
```

## User Feedback on Errors

### Toast Notifications

Show toast notifications for non-critical errors:

```jsx
// Example toast notification system
const ToastContext = createContext(null);

export const ToastProvider = ({ children }) => {
  const [toasts, setToasts] = useState([]);
  
  const addToast = useCallback((message, type = 'info', duration = 5000) => {
    const id = Date.now().toString();
    setToasts(prev => [...prev, { id, message, type, duration }]);
    
    if (duration > 0) {
      setTimeout(() => {
        removeToast(id);
      }, duration);
    }
    
    return id;
  }, []);
  
  const removeToast = useCallback((id) => {
    setToasts(prev => prev.filter(toast => toast.id !== id));
  }, []);
  
  const value = {
    toasts,
    addToast,
    removeToast
  };
  
  return (
    <ToastContext.Provider value={value}>
      {children}
      <ToastContainer />
    </ToastContext.Provider>
  );
};

// Usage
const { addToast } = useContext(ToastContext);

try {
  await saveTrack(trackData);
  addToast('Track saved successfully', 'success');
} catch (error) {
  addToast(`Failed to save track: ${error.message}`, 'error');
}
```

### Error Recovery Actions

Provide actionable recovery options for errors:

```jsx
// Example error recovery actions
const UploadErrorHandler = ({ error, file, onRetry, onCancel }) => {
  if (error.type === 'network') {
    return (
      <div className="error-actions">
        <p>Network error occurred during upload.</p>
        <div className="buttons">
          <Button onClick={onRetry}>Retry Upload</Button>
          <Button variant="secondary" onClick={onCancel}>Cancel</Button>
        </div>
      </div>
    );
  }
  
  if (error.type === 'validation') {
    return (
      <div className="error-actions">
        <p>File validation failed:</p>
        <ul>
          {Object.values(error.details).map((message, i) => (
            <li key={i}>{message}</li>
          ))}
        </ul>
        <div className="buttons">
          <Button onClick={onRetry}>Select Another File</Button>
          <Button variant="secondary" onClick={onCancel}>Cancel</Button>
        </div>
      </div>
    );
  }
  
  return (
    <div className="error-actions">
      <p>{error.message}</p>
      <div className="buttons">
        <Button onClick={onRetry}>Try Again</Button>
        <Button variant="secondary" onClick={onCancel}>Cancel</Button>
      </div>
    </div>
  );
};
```

## Debugging Tools

### Development Error Panel

Create a developer-focused error panel for debugging:

```jsx
// Example development error panel
const DevErrorPanel = ({ error }) => {
  if (process.env.NODE_ENV !== 'development') {
    return null;
  }
  
  return (
    <div className="dev-error-panel">
      <h3>Developer Error Information</h3>
      <div className="error-details">
        <div className="error-prop">
          <span className="label">Name:</span>
          <span className="value">{error.name}</span>
        </div>
        <div className="error-prop">
          <span className="label">Message:</span>
          <span className="value">{error.message}</span>
        </div>
        {error.code && (
          <div className="error-prop">
            <span className="label">Code:</span>
            <span className="value">{error.code}</span>
          </div>
        )}
        {error.status && (
          <div className="error-prop">
            <span className="label">Status:</span>
            <span className="value">{error.status}</span>
          </div>
        )}
        <div className="error-stack">
          <span className="label">Stack:</span>
          <pre>{error.stack}</pre>
        </div>
        {error.originalError && (
          <div className="error-cause">
            <span className="label">Cause:</span>
            <DevErrorPanel error={error.originalError} />
          </div>
        )}
      </div>
    </div>
  );
};
```

### Error Boundaries for Different Application Sections

Use different error boundaries for different application sections:

```jsx
// App entry point with section-specific error boundaries
const App = () => {
  return (
    <ErrorBoundary
      fallback={(error) => <CriticalErrorPage error={error} />}
      errorMetadata={{ section: 'root' }}
    >
      <ToastProvider>
        <Router>
          <AuthProvider>
            <Layout>
              <Switch>
                <Route path="/dashboard">
                  <ErrorBoundary
                    fallback={(error) => <SectionErrorPage title="Dashboard" error={error} />}
                    errorMetadata={{ section: 'dashboard' }}
                  >
                    <Dashboard />
                  </ErrorBoundary>
                </Route>
                <Route path="/tracks">
                  <ErrorBoundary
                    fallback={(error) => <SectionErrorPage title="Tracks Library" error={error} />}
                    errorMetadata={{ section: 'tracks' }}
                  >
                    <TracksSection />
                  </ErrorBoundary>
                </Route>
                {/* Other routes */}
              </Switch>
            </Layout>
          </AuthProvider>
        </Router>
      </ToastProvider>
    </ErrorBoundary>
  );
};
```

## Related Documents

- [Testing Strategy](testing-strategy.md)
- [API Endpoints Reference](../api/endpoints-reference.md)
- [Development Guide](development.md) 
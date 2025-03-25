# UI Component Guide

**Date**: [Current Date]

This document outlines the UI component architecture, design patterns, and implementation guidelines for the PCI File Manager application.

## UI Technology Stack

The frontend of the application is built using:

- **Electron**: Cross-platform desktop framework
- **React**: UI library for component-based development
- **TailwindCSS**: Utility-first CSS framework for styling
- **React Router**: For navigation between different views
- **React Query**: For data fetching, caching, and state management

## Component Hierarchy

The application UI is organized into a hierarchical component structure:

```
App
├── AppShell
│   ├── Sidebar
│   │   ├── Navigation
│   │   └── UserProfile
│   └── MainContent
│       ├── Header
│       │   ├── SearchBar
│       │   └── ActionButtons
│       └── PageContent
│           ├── Dashboard
│           ├── TrackList
│           │   └── TrackItem
│           ├── TrackDetail
│           │   ├── AudioPlayer
│           │   ├── MetadataEditor
│           │   └── WaveformDisplay
│           ├── UploadManager
│           │   ├── DropZone
│           │   ├── UploadQueue
│           │   └── ProcessingStatus
│           ├── PlaylistManager
│           └── SettingsPanel
└── Modals
    ├── LoginModal
    ├── ConfirmationModal
    └── ErrorModal
```

## Core Components

### AppShell

The main layout component that wraps the entire application.

```jsx
// Example AppShell component (pseudocode)
const AppShell = ({ children }) => {
  const { isAuthenticated } = useAuth();
  
  if (!isAuthenticated) {
    return <LoginScreen />;
  }
  
  return (
    <div className="flex h-screen bg-gray-100 dark:bg-gray-900">
      <Sidebar />
      <div className="flex flex-col flex-1 overflow-hidden">
        <Header />
        <main className="flex-1 overflow-y-auto p-4">
          {children}
        </main>
        <StatusBar />
      </div>
      <ModalContainer />
    </div>
  );
};
```

### TrackList

Displays a paginated list of tracks with sorting and filtering options.

```jsx
// Example TrackList component (pseudocode)
const TrackList = () => {
  const [filters, setFilters] = useState({
    search: '',
    artist: '',
    album: '',
    genre: '',
  });
  
  const [sort, setSort] = useState({ field: 'dateAdded', order: 'desc' });
  const [page, setPage] = useState(1);
  
  const { data, isLoading, error } = useQuery(
    ['tracks', filters, sort, page],
    () => fetchTracks({ ...filters, ...sort, page })
  );
  
  if (isLoading) return <LoadingSpinner />;
  if (error) return <ErrorMessage error={error} />;
  
  return (
    <div className="space-y-4">
      <div className="flex justify-between">
        <FilterBar filters={filters} onChange={setFilters} />
        <SortSelector value={sort} onChange={setSort} />
      </div>
      
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
        <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
          <thead className="bg-gray-50 dark:bg-gray-700">
            {/* Table headers */}
          </thead>
          <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
            {data.tracks.map(track => (
              <TrackItem key={track.id} track={track} />
            ))}
          </tbody>
        </table>
      </div>
      
      <Pagination
        currentPage={page}
        totalPages={data.pagination.pages}
        onPageChange={setPage}
      />
    </div>
  );
};
```

### UploadManager

Handles file uploads with drag-and-drop support and progress tracking.

```jsx
// Example UploadManager component (pseudocode)
const UploadManager = () => {
  const [files, setFiles] = useState([]);
  const [activeUploads, setActiveUploads] = useState({});
  
  const onDrop = useCallback(acceptedFiles => {
    // Process dropped files
    const newFiles = acceptedFiles.map(file => ({
      id: generateId(),
      file,
      progress: 0,
      status: 'pending',
      metadata: {
        title: file.name.replace(/\.[^/.]+$/, ""),
        artist: '',
        album: ''
      }
    }));
    
    setFiles(prev => [...prev, ...newFiles]);
  }, []);
  
  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'audio/mpeg': ['.mp3'],
      'audio/wav': ['.wav'],
      'audio/flac': ['.flac'],
      'audio/aac': ['.aac', '.m4a']
    },
    maxSize: 1024 * 1024 * 500 // 500MB
  });
  
  const uploadFile = async (fileItem) => {
    try {
      // Initiate upload process
      const uploadData = await initiateUpload(fileItem.file);
      
      // Upload file with progress tracking
      const upload = new Upload({
        client: r2Client,
        params: { /* upload params */ }
      });
      
      upload.on('httpUploadProgress', (progress) => {
        setActiveUploads(prev => ({
          ...prev,
          [fileItem.id]: {
            ...prev[fileItem.id],
            progress: Math.round((progress.loaded / progress.total) * 100)
          }
        }));
      });
      
      await upload.done();
      
      // Process file after upload
      const result = await completeUpload(uploadData.uploadId);
      
      // Update state with result
      setFiles(prev => prev.map(f => 
        f.id === fileItem.id 
          ? { ...f, status: 'completed', trackId: result.trackId } 
          : f
      ));
      
    } catch (error) {
      console.error('Upload failed:', error);
      setFiles(prev => prev.map(f => 
        f.id === fileItem.id 
          ? { ...f, status: 'error', error: error.message } 
          : f
      ));
    }
  };
  
  return (
    <div className="space-y-6">
      <div 
        {...getRootProps()} 
        className={`border-2 border-dashed rounded-lg p-12 text-center cursor-pointer transition ${
          isDragActive ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20' : 'border-gray-300 dark:border-gray-700'
        }`}
      >
        <input {...getInputProps()} />
        <div className="space-y-2">
          <CloudUploadIcon className="h-12 w-12 mx-auto text-gray-400" />
          <p className="text-lg">Drag & drop audio files here, or click to select files</p>
          <p className="text-sm text-gray-500 dark:text-gray-400">Supports MP3, WAV, FLAC, AAC (max 500MB)</p>
        </div>
      </div>
      
      {files.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
          <div className="p-4 border-b dark:border-gray-700">
            <h3 className="text-lg font-medium">Upload Queue</h3>
          </div>
          <ul className="divide-y divide-gray-200 dark:divide-gray-700">
            {files.map(fileItem => (
              <UploadItem 
                key={fileItem.id} 
                fileItem={fileItem} 
                onUpload={() => uploadFile(fileItem)}
                onRemove={() => removeFile(fileItem.id)}
                onEdit={(metadata) => updateFileMetadata(fileItem.id, metadata)}
              />
            ))}
          </ul>
        </div>
      )}
    </div>
  );
};
```

## Reusable Components

### Button

A customizable button component with different variants.

```jsx
// Example Button component (pseudocode)
const Button = ({ 
  children, 
  variant = 'primary', 
  size = 'md', 
  isLoading = false, 
  disabled = false, 
  className = '', 
  onClick, 
  ...props 
}) => {
  const baseClasses = "inline-flex items-center justify-center rounded-md font-medium focus:outline-none focus:ring-2 focus:ring-offset-2";
  
  const variantClasses = {
    primary: "bg-blue-600 hover:bg-blue-700 text-white focus:ring-blue-500",
    secondary: "bg-gray-200 hover:bg-gray-300 text-gray-900 focus:ring-gray-500 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-white",
    danger: "bg-red-600 hover:bg-red-700 text-white focus:ring-red-500",
    ghost: "bg-transparent hover:bg-gray-100 text-gray-700 dark:hover:bg-gray-800 dark:text-gray-300",
  };
  
  const sizeClasses = {
    sm: "px-3 py-1.5 text-sm",
    md: "px-4 py-2",
    lg: "px-6 py-3 text-lg",
  };
  
  const allClasses = [
    baseClasses,
    variantClasses[variant],
    sizeClasses[size],
    disabled || isLoading ? "opacity-50 cursor-not-allowed" : "",
    className
  ].join(" ");
  
  return (
    <button
      className={allClasses}
      disabled={disabled || isLoading}
      onClick={onClick}
      {...props}
    >
      {isLoading && (
        <svg className="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      )}
      {children}
    </button>
  );
};
```

### Card

A versatile card component for content containers.

```jsx
// Example Card component (pseudocode)
const Card = ({ 
  children, 
  title, 
  footer, 
  className = '' 
}) => {
  return (
    <div className={`bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden ${className}`}>
      {title && (
        <div className="px-4 py-3 border-b dark:border-gray-700">
          <h3 className="text-lg font-medium">{title}</h3>
        </div>
      )}
      <div className="p-4">
        {children}
      </div>
      {footer && (
        <div className="px-4 py-3 bg-gray-50 dark:bg-gray-750 border-t dark:border-gray-700">
          {footer}
        </div>
      )}
    </div>
  );
};
```

### Modal

A reusable modal dialog component.

```jsx
// Example Modal component (pseudocode)
const Modal = ({ 
  isOpen, 
  onClose, 
  title, 
  children, 
  footer,
  size = 'md'
}) => {
  const sizeClasses = {
    sm: 'max-w-md',
    md: 'max-w-lg',
    lg: 'max-w-2xl',
    xl: 'max-w-4xl',
    full: 'max-w-full mx-4'
  };
  
  if (!isOpen) return null;
  
  return (
    <div className="fixed inset-0 z-50 overflow-y-auto">
      <div className="flex items-center justify-center min-h-screen p-4">
        <div className="fixed inset-0 bg-black bg-opacity-50 transition-opacity" onClick={onClose}></div>
        
        <div className={`relative bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full ${sizeClasses[size]} z-10`}>
          {title && (
            <div className="px-6 py-4 border-b dark:border-gray-700">
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-medium">{title}</h3>
                <button 
                  type="button" 
                  onClick={onClose}
                  className="text-gray-400 hover:text-gray-500 focus:outline-none"
                >
                  <span className="sr-only">Close</span>
                  <XIcon className="h-6 w-6" />
                </button>
              </div>
            </div>
          )}
          
          <div className="px-6 py-4">
            {children}
          </div>
          
          {footer && (
            <div className="px-6 py-4 bg-gray-50 dark:bg-gray-750 border-t dark:border-gray-700 flex justify-end space-x-3">
              {footer}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
```

## State Management

### Context-Based State

For global application state, we use React Context:

```jsx
// Example AuthContext (pseudocode)
const AuthContext = createContext(null);

export const AuthProvider = ({ children }) => {
  const [user, setUser] = useState(null);
  const [isLoading, setIsLoading] = useState(true);
  
  useEffect(() => {
    // Check for existing tokens and validate on load
    const checkAuth = async () => {
      try {
        const token = localStorage.getItem('accessToken');
        if (!token) {
          setIsLoading(false);
          return;
        }
        
        const userData = await validateToken(token);
        setUser(userData);
      } catch (error) {
        console.error('Auth error:', error);
        localStorage.removeItem('accessToken');
        localStorage.removeItem('refreshToken');
      } finally {
        setIsLoading(false);
      }
    };
    
    checkAuth();
  }, []);
  
  const login = async (credentials) => {
    const response = await apiClient.post('/auth/login', credentials);
    const { token, refreshToken, user } = response.data;
    
    localStorage.setItem('accessToken', token);
    localStorage.setItem('refreshToken', refreshToken);
    setUser(user);
    
    return user;
  };
  
  const logout = async () => {
    try {
      await apiClient.post('/auth/logout');
    } catch (error) {
      console.error('Logout error:', error);
    } finally {
      localStorage.removeItem('accessToken');
      localStorage.removeItem('refreshToken');
      setUser(null);
    }
  };
  
  const refreshAuth = async () => {
    const refreshToken = localStorage.getItem('refreshToken');
    if (!refreshToken) {
      throw new Error('No refresh token');
    }
    
    const response = await apiClient.post('/auth/refresh', { refreshToken });
    const { token } = response.data;
    
    localStorage.setItem('accessToken', token);
    return token;
  };
  
  const value = {
    user,
    isAuthenticated: !!user,
    isLoading,
    login,
    logout,
    refreshAuth
  };
  
  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};
```

### Data Fetching with React Query

For server state, we use React Query:

```jsx
// Example of React Query usage (pseudocode)
const useTracks = (filters, sort, page) => {
  return useQuery(
    ['tracks', filters, sort, page], 
    () => fetchTracks({ ...filters, ...sort, page }),
    {
      keepPreviousData: true,
      staleTime: 60000, // 1 minute
      refetchOnWindowFocus: true
    }
  );
};

const useTrack = (trackId) => {
  return useQuery(
    ['track', trackId],
    () => fetchTrack(trackId),
    {
      enabled: !!trackId,
      staleTime: 300000 // 5 minutes
    }
  );
};

// Using the hooks
const TrackListPage = () => {
  const [filters, setFilters] = useState({});
  const [sort, setSort] = useState({ field: 'dateAdded', order: 'desc' });
  const [page, setPage] = useState(1);
  
  const { data, isLoading, error } = useTracks(filters, sort, page);
  
  // Component rendering
};
```

## Theme and Styling

The application uses a dark/light theme system built on TailwindCSS:

```jsx
// Example ThemeContext (pseudocode)
const ThemeContext = createContext(null);

export const ThemeProvider = ({ children }) => {
  const [theme, setTheme] = useState(() => {
    const savedTheme = localStorage.getItem('theme');
    return savedTheme || (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
  });
  
  useEffect(() => {
    if (theme === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
    localStorage.setItem('theme', theme);
  }, [theme]);
  
  const toggleTheme = () => {
    setTheme(prevTheme => (prevTheme === 'light' ? 'dark' : 'light'));
  };
  
  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      {children}
    </ThemeContext.Provider>
  );
};

export const useTheme = () => useContext(ThemeContext);
```

## Form Handling

We use Formik for complex forms with validation:

```jsx
// Example MetadataForm component (pseudocode)
const MetadataForm = ({ track, onSubmit, isSubmitting }) => {
  const validationSchema = Yup.object({
    title: Yup.string().required('Title is required'),
    artist: Yup.string().required('Artist is required'),
    album: Yup.string(),
    genre: Yup.array().of(Yup.string()),
    releaseYear: Yup.number().min(1900).max(new Date().getFullYear()),
    isPublic: Yup.boolean()
  });
  
  return (
    <Formik
      initialValues={{
        title: track?.title || '',
        artist: track?.artist || '',
        album: track?.album || '',
        genre: track?.genre || [],
        releaseYear: track?.releaseYear || new Date().getFullYear(),
        isPublic: track?.isPublic ?? true
      }}
      validationSchema={validationSchema}
      onSubmit={onSubmit}
    >
      {({ values, errors, touched, handleChange, handleBlur, setFieldValue }) => (
        <Form className="space-y-6">
          <div className="space-y-4">
            <FormField
              label="Title"
              name="title"
              type="text"
              error={touched.title && errors.title}
            />
            
            <FormField
              label="Artist"
              name="artist"
              type="text"
              error={touched.artist && errors.artist}
            />
            
            <FormField
              label="Album"
              name="album"
              type="text"
              error={touched.album && errors.album}
            />
            
            <div>
              <label className="block text-sm font-medium mb-1">Genre</label>
              <TagInput
                value={values.genre}
                onChange={(tags) => setFieldValue('genre', tags)}
                suggestions={commonGenres}
                placeholder="Add genres..."
              />
            </div>
            
            <FormField
              label="Release Year"
              name="releaseYear"
              type="number"
              min={1900}
              max={new Date().getFullYear()}
              error={touched.releaseYear && errors.releaseYear}
            />
            
            <div className="flex items-center">
              <Switch
                checked={values.isPublic}
                onChange={(checked) => setFieldValue('isPublic', checked)}
                className="mr-2"
              />
              <label className="text-sm font-medium">Public track</label>
            </div>
          </div>
          
          <div className="flex justify-end space-x-3">
            <Button variant="secondary" type="button" onClick={onCancel}>
              Cancel
            </Button>
            <Button variant="primary" type="submit" isLoading={isSubmitting}>
              Save Changes
            </Button>
          </div>
        </Form>
      )}
    </Formik>
  );
};
```

## Accessibility Guidelines

### Keyboard Navigation

All interactive elements must be accessible via keyboard:

```jsx
// Example accessible button (pseudocode)
const IconButton = ({ 
  icon, 
  label, 
  onClick, 
  disabled = false 
}) => {
  return (
    <button
      aria-label={label}
      title={label}
      onClick={onClick}
      disabled={disabled}
      className="p-2 rounded-full hover:bg-gray-100 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
    >
      {icon}
    </button>
  );
};
```

### Screen Reader Support

Use proper ARIA attributes for screen reader accessibility:

```jsx
// Example Modal with ARIA support (pseudocode)
const Modal = ({ isOpen, onClose, title, children }) => {
  const modalRef = useRef(null);
  
  useEffect(() => {
    if (isOpen) {
      // Focus the modal when it opens
      modalRef.current?.focus();
      // Trap focus inside modal
      // ...focus trap implementation
    }
  }, [isOpen]);
  
  if (!isOpen) return null;
  
  return (
    <div 
      className="modal-overlay"
      aria-modal="true"
      role="dialog"
      aria-labelledby="modal-title"
    >
      <div 
        ref={modalRef}
        className="modal-content"
        tabIndex="-1"
      >
        <h2 id="modal-title">{title}</h2>
        <div>{children}</div>
        <button 
          aria-label="Close modal"
          onClick={onClose}
        >
          Close
        </button>
      </div>
    </div>
  );
};
```

### Color Contrast

Ensure sufficient color contrast for all text:

- Regular text: minimum contrast ratio of 4.5:1
- Large text: minimum contrast ratio of 3:1
- UI components and graphics: minimum contrast ratio of 3:1

Use Tailwind's color palette which is designed with accessibility in mind, and test with tools like [WebAIM's Contrast Checker](https://webaim.org/resources/contrastchecker/).

### Focus Management

Maintain clear focus indicators:

```css
/* Custom focus styles */
.focus-ring {
  @apply focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-blue-400 dark:focus:ring-offset-gray-900;
}
```

## Electron-Specific Components

### Main Process Communication

Components for IPC with the Electron main process:

```jsx
// Example useIPC hook (pseudocode)
const useIPC = () => {
  const sendToMain = useCallback((channel, data) => {
    window.electron.send(channel, data);
  }, []);
  
  const listenToMain = useCallback((channel, callback) => {
    const listener = (event, ...args) => callback(...args);
    window.electron.on(channel, listener);
    
    return () => {
      window.electron.removeListener(channel, listener);
    };
  }, []);
  
  return { sendToMain, listenToMain };
};

// Example usage
const FileSystemPicker = ({ onFileSelect }) => {
  const { sendToMain, listenToMain } = useIPC();
  
  useEffect(() => {
    const cleanup = listenToMain('file-selected', (filePath) => {
      onFileSelect(filePath);
    });
    
    return cleanup;
  }, [onFileSelect, listenToMain]);
  
  const handleClick = () => {
    sendToMain('open-file-dialog', { 
      title: 'Select Audio File',
      filters: [
        { name: 'Audio Files', extensions: ['mp3', 'wav', 'flac', 'aac', 'm4a'] }
      ]
    });
  };
  
  return (
    <Button onClick={handleClick}>
      Browse Files
    </Button>
  );
};
```

### Native Features Integration

Components that interface with native OS features:

```jsx
// Example system notifications (pseudocode)
const useNotification = () => {
  const showNotification = useCallback(({ title, body, onClick }) => {
    // Check if we're in Electron
    if (window.electron) {
      window.electron.send('show-notification', { title, body });
      
      if (onClick) {
        const cleanup = window.electron.on('notification-clicked', onClick);
        return cleanup;
      }
    } else {
      // Fallback to web notifications
      if (Notification.permission === 'granted') {
        const notification = new Notification(title, { body });
        
        if (onClick) {
          notification.onclick = onClick;
        }
      }
    }
  }, []);
  
  return { showNotification };
};

// Example usage
const UploadCompleteNotification = ({ track }) => {
  const { showNotification } = useNotification();
  const navigate = useNavigate();
  
  useEffect(() => {
    showNotification({
      title: 'Upload Complete',
      body: `"${track.title}" by ${track.artist} has been successfully uploaded.`,
      onClick: () => navigate(`/tracks/${track.id}`)
    });
  }, [track, showNotification, navigate]);
  
  return null; // No UI, just shows a notification
};
```

## Component Testing Patterns

Best practices for testing components:

```jsx
// Example test for Button component (pseudocode)
describe('Button Component', () => {
  it('renders correctly with default props', () => {
    render(<Button>Click Me</Button>);
    
    const button = screen.getByRole('button', { name: /click me/i });
    expect(button).toBeInTheDocument();
    expect(button).toHaveClass('bg-blue-600'); // primary variant
  });
  
  it('applies different variants correctly', () => {
    render(<Button variant="secondary">Secondary</Button>);
    
    const button = screen.getByRole('button', { name: /secondary/i });
    expect(button).toHaveClass('bg-gray-200');
    expect(button).not.toHaveClass('bg-blue-600');
  });
  
  it('handles click events', () => {
    const handleClick = jest.fn();
    render(<Button onClick={handleClick}>Click Me</Button>);
    
    const button = screen.getByRole('button', { name: /click me/i });
    userEvent.click(button);
    
    expect(handleClick).toHaveBeenCalledTimes(1);
  });
  
  it('shows loading state correctly', () => {
    render(<Button isLoading>Loading</Button>);
    
    const button = screen.getByRole('button', { name: /loading/i });
    expect(button).toBeDisabled();
    expect(button).toHaveClass('opacity-50');
    
    // Check for spinner
    const spinner = document.querySelector('svg.animate-spin');
    expect(spinner).toBeInTheDocument();
  });
});
```

## Related Documents

- [Development Roadmap](development-roadmap.md)
- [API Endpoints Reference](../api/endpoints-reference.md)
- [Authentication Flow](../architecture/authentication-flow.md) 
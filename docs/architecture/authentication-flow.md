# Authentication & Authorization Flow

**Date**: [Current Date]

This document details the authentication and authorization architecture for the PCI File Manager application.

## Authentication Architecture

The application uses a JWT (JSON Web Token) based authentication system to secure access to the system and its resources.

### Authentication Flow

```
┌─────────┐                ┌─────────────┐                ┌─────────────┐
│  Client │                │  API Server │                │  MongoDB    │
└────┬────┘                └──────┬──────┘                └──────┬──────┘
     │                            │                              │
     │   1. Login Request         │                              │
     │ ─────────────────────────> │                              │
     │                            │                              │
     │                            │  2. Verify Credentials       │
     │                            │ ─────────────────────────────>
     │                            │                              │
     │                            │  3. Return User Record       │
     │                            │ <─────────────────────────────
     │                            │                              │
     │                            │  4. Generate JWT             │
     │                            │ ──────────┐                  │
     │                            │           │                  │
     │                            │ <─────────┘                  │
     │                            │                              │
     │   5. Return JWT            │                              │
     │ <─────────────────────────────                            │
     │                            │                              │
     │   6. Store Token Locally   │                              │
     │ ──────┐                    │                              │
     │       │                    │                              │
     │ <─────┘                    │                              │
     │                            │                              │
     │   7. API Request with JWT  │                              │
     │ ─────────────────────────> │                              │
     │                            │                              │
     │                            │  8. Validate JWT            │
     │                            │ ──────────┐                  │
     │                            │           │                  │
     │                            │ <─────────┘                  │
     │                            │                              │
     │   9. API Response          │                              │
     │ <─────────────────────────────                            │
     │                            │                              │
```

#### Steps in Detail:

1. **Login Request**: Client submits credentials (username/email and password)
2. **Verify Credentials**: Server queries MongoDB for the user record
3. **User Lookup**: MongoDB returns the user record if found
4. **JWT Generation**: Server creates a signed JWT containing user identity and permissions
5. **Return JWT**: Server returns the JWT to the client
6. **Store Token**: Client stores the JWT (in memory and secure storage)
7. **API Requests**: Client includes the JWT in the Authorization header for subsequent requests
8. **Validate JWT**: Server validates the token for each request
9. **API Response**: Server returns requested data if token is valid

### Token Structure

```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    "sub": "<user_id>",
    "iat": <timestamp>,
    "exp": <expiration_timestamp>,
    "username": "<username>",
    "email": "<email>",
    "role": "<role>",
    "permissions": {
      "canUpload": true,
      "canEdit": true,
      "canDelete": false,
      "canManageUsers": false
    }
  },
  "signature": "..."
}
```

### Token Management

#### Token Generation

```javascript
// Example token generation (pseudocode)
function generateToken(user) {
  const payload = {
    sub: user._id,
    iat: Math.floor(Date.now() / 1000),
    exp: Math.floor(Date.now() / 1000) + (60 * 60), // 1 hour expiration
    username: user.username,
    email: user.email,
    role: user.role,
    permissions: user.permissions
  };
  
  return jwt.sign(payload, JWT_SECRET, { algorithm: 'HS256' });
}
```

#### Token Validation

```javascript
// Example token validation middleware (pseudocode)
function authenticateToken(req, res, next) {
  const authHeader = req.headers['authorization'];
  const token = authHeader && authHeader.split(' ')[1];
  
  if (!token) return res.sendStatus(401);
  
  jwt.verify(token, JWT_SECRET, (err, user) => {
    if (err) return res.sendStatus(403);
    req.user = user;
    next();
  });
}
```

#### Token Refresh

To prevent users from having to re-login frequently, a token refresh mechanism is implemented:

1. The client receives both an access token (short-lived) and a refresh token (longer-lived)
2. When the access token expires, the client uses the refresh token to obtain a new access token
3. If the refresh token is expired, the user must re-authenticate

```javascript
// Example refresh token function (pseudocode)
function refreshToken(req, res) {
  const refreshToken = req.body.token;
  
  if (!refreshToken) return res.sendStatus(401);
  
  // Verify the refresh token (store in a separate Redis or MongoDB collection)
  jwt.verify(refreshToken, REFRESH_TOKEN_SECRET, (err, user) => {
    if (err) return res.sendStatus(403);
    
    // Generate new access token
    const accessToken = generateAccessToken(user);
    res.json({ accessToken });
  });
}
```

### Token Storage

#### Client-Side Storage

- **Access Token**: Stored in memory or sessionStorage (non-persistent)
- **Refresh Token**: Stored in an HTTP-only cookie or secure localStorage with additional encryption

#### Server-Side Storage

- **Active Refresh Tokens**: Stored in MongoDB or Redis
- **Revoked Tokens**: Blacklist for invalidated tokens before expiration

## Authorization System

### Role-Based Access Control (RBAC)

The application uses a role-based system with predefined roles:

1. **Admin**: Full system access
2. **Editor**: Can upload and edit content, but not manage users
3. **Viewer**: Can only view and search content, no edit capabilities

### Permission Matrix

| Resource/Action | Admin | Editor | Viewer |
|----------------|-------|--------|--------|
| View Tracks    | ✓     | ✓      | ✓      |
| Upload Tracks  | ✓     | ✓      | ✗      |
| Edit Metadata  | ✓     | ✓      | ✗      |
| Delete Tracks  | ✓     | ✓      | ✗      |
| Manage Users   | ✓     | ✗      | ✗      |
| System Config  | ✓     | ✗      | ✗      |

### Permission Enforcement

Permissions are enforced at multiple levels:

1. **UI Level**: Hide/disable functionality based on user role
2. **API Level**: Middleware checks before processing requests
3. **Database Level**: MongoDB read/write permissions

```javascript
// Example authorization middleware (pseudocode)
function authorizeAction(requiredPermission) {
  return (req, res, next) => {
    if (!req.user) return res.sendStatus(401);
    
    // Check if user has the required permission
    if (req.user.permissions[requiredPermission] || req.user.role === 'admin') {
      next();
    } else {
      res.status(403).json({ message: "Insufficient permissions" });
    }
  };
}

// Usage in routes
app.post('/api/tracks', authenticateToken, authorizeAction('canUpload'), createTrack);
```

## Security Considerations

### Password Handling

1. **Hashing**: Passwords are hashed using bcrypt with appropriate salt rounds
2. **Storage**: Only password hashes are stored, never plaintext passwords
3. **Validation**: Rate limiting on login attempts to prevent brute force attacks

```javascript
// Example password hashing (pseudocode)
async function hashPassword(password) {
  const saltRounds = 12;
  return bcrypt.hash(password, saltRounds);
}

// Example password verification (pseudocode)
async function verifyPassword(password, hashedPassword) {
  return bcrypt.compare(password, hashedPassword);
}
```

### JWT Security

1. **Expiration**: Short-lived access tokens (1 hour max)
2. **Secret Key**: Strong, unique secret key for signing
3. **Algorithm**: HS256 or stronger signing algorithm
4. **Claims**: Minimal payload information

### HTTPS

All API communication must use HTTPS to prevent token theft through man-in-the-middle attacks.

### CORS Configuration

Proper CORS configuration to restrict API access to approved origins:

```javascript
// Example CORS configuration (pseudocode)
const corsOptions = {
  origin: ['https://app.example.com', 'https://admin.example.com'],
  methods: ['GET', 'POST', 'PUT', 'DELETE'],
  allowedHeaders: ['Content-Type', 'Authorization'],
  credentials: true,
  maxAge: 86400 // 24 hours
};

app.use(cors(corsOptions));
```

### XSS Protection

1. **Content Security Policy**: Implement strict CSP headers
2. **Input Validation**: Sanitize all user inputs
3. **Output Encoding**: Encode dynamic content before rendering

### CSRF Protection

For cookie-based refresh tokens, implement CSRF protection:

1. **CSRF Tokens**: Generate per-session tokens for state-changing operations
2. **SameSite Cookies**: Set SameSite=Strict for authentication cookies

## Session Management

### Session Tracking

1. **Login Tracking**: Record login timestamp, IP, and device information
2. **Active Sessions**: Allow users to view and terminate active sessions
3. **Inactivity Timeout**: Automatically expire sessions after inactivity period (30 minutes)

### Session Termination

1. **Logout**: Delete client-side tokens and add refresh token to blacklist
2. **Force Logout**: Admin ability to terminate user sessions remotely
3. **Password Change**: Invalidate all existing sessions when password is changed

## Integration with Electron

Since this is an Electron application, additional considerations include:

1. **IPC Security**: Secure communication between main and renderer processes
2. **Token Storage**: Use Electron's secure storage options (keychain/credential vault)
3. **Auto-Login**: Optional remembering of credentials with secure storage

## Related Documents

- [Database Schema](database-schema.md)
- [API Endpoints Reference](../api/endpoints-reference.md)
- [Development Guide](../guides/development.md) 
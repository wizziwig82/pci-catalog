# User Management Guide

*Last Updated: March 25, 2024*

This guide provides comprehensive instructions for implementing and managing user accounts, authentication, authorization, and role-based access control in the PCI File Manager application.

## Overview

The user management system in PCI File Manager includes:
- User registration and authentication
- Role-based access control (RBAC)
- Permission management
- User profile management
- Session handling
- Security best practices

## User Authentication System

### Authentication Flow

The application implements a secure authentication system:

1. User enters credentials (email/username and password)
2. Credentials are validated against the database (passwords are hashed)
3. Upon successful authentication, JWT tokens are issued:
   - Access token (short-lived)
   - Refresh token (longer-lived, for obtaining new access tokens)
4. Tokens are used to authenticate subsequent API requests

More details are available in the [Authentication Flow](../architecture/authentication-flow.md) document.

### Authentication Implementation

#### User Login Process

```javascript
// src/services/auth-service.js
const bcrypt = require('bcrypt');
const jwt = require('jsonwebtoken');
const UserModel = require('../models/user-model');

class AuthService {
  async login(email, password) {
    // Find user by email
    const user = await UserModel.findOne({ email });
    if (!user) {
      throw new Error('User not found');
    }
    
    // Verify password
    const isPasswordValid = await bcrypt.compare(password, user.passwordHash);
    if (!isPasswordValid) {
      throw new Error('Invalid password');
    }
    
    // Generate tokens
    const accessToken = this.generateAccessToken(user);
    const refreshToken = this.generateRefreshToken(user);
    
    // Update refresh token in database
    await user.updateOne({ refreshToken: refreshToken });
    
    return {
      accessToken,
      refreshToken,
      user: {
        id: user._id,
        email: user.email,
        name: user.name,
        role: user.role
      }
    };
  }
  
  generateAccessToken(user) {
    return jwt.sign(
      { 
        userId: user._id,
        email: user.email,
        role: user.role
      },
      process.env.JWT_ACCESS_SECRET,
      { expiresIn: '15m' }
    );
  }
  
  generateRefreshToken(user) {
    return jwt.sign(
      { userId: user._id },
      process.env.JWT_REFRESH_SECRET,
      { expiresIn: '7d' }
    );
  }
  
  // Additional authentication methods
  // ...
}

module.exports = new AuthService();
```

#### Token Verification Middleware

```javascript
// src/middlewares/auth-middleware.js
const jwt = require('jsonwebtoken');

function authMiddleware(req, res, next) {
  try {
    // Get token from Authorization header
    const authHeader = req.headers.authorization;
    if (!authHeader) {
      return res.status(401).json({ message: 'Authorization header missing' });
    }
    
    const token = authHeader.split(' ')[1]; // "Bearer <token>"
    if (!token) {
      return res.status(401).json({ message: 'Authentication token missing' });
    }
    
    // Verify token
    const decoded = jwt.verify(token, process.env.JWT_ACCESS_SECRET);
    
    // Add user data to request
    req.user = {
      userId: decoded.userId,
      email: decoded.email,
      role: decoded.role
    };
    
    next();
  } catch (error) {
    if (error instanceof jwt.TokenExpiredError) {
      return res.status(401).json({ message: 'Token expired' });
    }
    if (error instanceof jwt.JsonWebTokenError) {
      return res.status(401).json({ message: 'Invalid token' });
    }
    return res.status(500).json({ message: 'Internal server error' });
  }
}

module.exports = authMiddleware;
```

## User Registration

### Registration Process

1. Collect user information (name, email, password)
2. Validate inputs (email format, password strength)
3. Check if email already exists
4. Hash password using bcrypt
5. Create user record in database
6. Send verification email (optional)
7. Assign default role and permissions

### Implementation

```javascript
// src/services/user-service.js
const bcrypt = require('bcrypt');
const UserModel = require('../models/user-model');

class UserService {
  async registerUser(userData) {
    const { name, email, password } = userData;
    
    // Check if user already exists
    const existingUser = await UserModel.findOne({ email });
    if (existingUser) {
      throw new Error('User with this email already exists');
    }
    
    // Hash password
    const saltRounds = 10;
    const passwordHash = await bcrypt.hash(password, saltRounds);
    
    // Create new user with default role
    const newUser = new UserModel({
      name,
      email,
      passwordHash,
      role: 'user',
      isActive: true,
      createdAt: new Date(),
      lastLogin: null
    });
    
    // Save user to database
    await newUser.save();
    
    return {
      id: newUser._id,
      name: newUser.name,
      email: newUser.email,
      role: newUser.role
    };
  }
  
  // Additional user management methods
  // ...
}

module.exports = new UserService();
```

## Role-Based Access Control (RBAC)

### User Roles

The application implements the following user roles:

1. **Admin**: Full system access, including user management
2. **Manager**: Manages files, shares, and has limited user management capabilities
3. **User**: Standard user with access to own files and shared resources
4. **Viewer**: Read-only access to specific shared resources

### Role Definitions

Each role has associated permissions defined in the database:

```javascript
// Example role definition in MongoDB
{
  _id: ObjectId("..."),
  name: "manager",
  displayName: "Manager",
  description: "Can manage files and has limited user management capabilities",
  permissions: [
    "file:create",
    "file:read",
    "file:update",
    "file:delete",
    "file:share",
    "user:read",
    "user:create"
    // More permissions...
  ],
  createdAt: ISODate("2024-03-25...")
}
```

### Permission Checking Middleware

```javascript
// src/middlewares/permission-middleware.js
function checkPermission(requiredPermission) {
  return async (req, res, next) => {
    try {
      // Get user and role from request (set by auth middleware)
      const { userId, role } = req.user;
      
      // Fetch role permissions from database or cache
      const roleData = await getRoleWithPermissions(role);
      
      if (!roleData || !roleData.permissions.includes(requiredPermission)) {
        return res.status(403).json({ message: 'Permission denied' });
      }
      
      next();
    } catch (error) {
      return res.status(500).json({ message: 'Internal server error' });
    }
  };
}

module.exports = { checkPermission };
```

### Using Permission Middleware in Routes

```javascript
// src/routes/file-routes.js
const express = require('express');
const authMiddleware = require('../middlewares/auth-middleware');
const { checkPermission } = require('../middlewares/permission-middleware');
const FileController = require('../controllers/file-controller');

const router = express.Router();

// Apply authentication middleware to all routes
router.use(authMiddleware);

// Routes with permission checks
router.post('/', checkPermission('file:create'), FileController.createFile);
router.get('/', checkPermission('file:read'), FileController.listFiles);
router.get('/:id', checkPermission('file:read'), FileController.getFile);
router.put('/:id', checkPermission('file:update'), FileController.updateFile);
router.delete('/:id', checkPermission('file:delete'), FileController.deleteFile);
router.post('/:id/share', checkPermission('file:share'), FileController.shareFile);

module.exports = router;
```

## User Profile Management

### Profile Data Structure

User profiles store various information:

```javascript
// src/models/user-model.js
const mongoose = require('mongoose');

const userSchema = new mongoose.Schema({
  name: {
    type: String,
    required: true,
    trim: true
  },
  email: {
    type: String,
    required: true,
    unique: true,
    trim: true,
    lowercase: true
  },
  passwordHash: {
    type: String,
    required: true
  },
  role: {
    type: String,
    required: true,
    enum: ['admin', 'manager', 'user', 'viewer'],
    default: 'user'
  },
  profilePicture: {
    type: String,
    default: null
  },
  preferences: {
    theme: {
      type: String,
      enum: ['light', 'dark', 'system'],
      default: 'system'
    },
    language: {
      type: String,
      default: 'en'
    },
    notifications: {
      email: {
        type: Boolean,
        default: true
      },
      app: {
        type: Boolean,
        default: true
      }
    }
  },
  isActive: {
    type: Boolean,
    default: true
  },
  refreshToken: String,
  passwordResetToken: String,
  passwordResetExpires: Date,
  lastLogin: Date,
  createdAt: {
    type: Date,
    default: Date.now
  },
  updatedAt: {
    type: Date,
    default: Date.now
  }
});

// Middleware to update the 'updatedAt' field on save
userSchema.pre('save', function(next) {
  this.updatedAt = new Date();
  next();
});

const User = mongoose.model('User', userSchema);
module.exports = User;
```

### Profile Update Implementation

```javascript
// src/services/user-service.js (continued)
async updateUserProfile(userId, profileData) {
  // Validate user exists
  const user = await UserModel.findById(userId);
  if (!user) {
    throw new Error('User not found');
  }
  
  // Fields that can be updated by the user
  const allowedUpdates = ['name', 'profilePicture', 'preferences'];
  const updates = {};
  
  // Filter out only allowed fields
  Object.keys(profileData).forEach(key => {
    if (allowedUpdates.includes(key)) {
      updates[key] = profileData[key];
    }
  });
  
  // Update user
  await UserModel.findByIdAndUpdate(userId, updates);
  
  // Return updated user
  return await UserModel.findById(userId, {
    passwordHash: 0,
    refreshToken: 0
  });
}
```

## Password Management

### Password Reset Flow

1. User requests password reset via email
2. System generates reset token and stores in user record
3. Email is sent with password reset link
4. User clicks link and enters new password
5. System verifies token and updates password

### Implementation

```javascript
// src/services/auth-service.js (continued)
const crypto = require('crypto');
const sendEmail = require('../utils/email-sender');

async function requestPasswordReset(email) {
  // Find user
  const user = await UserModel.findOne({ email });
  if (!user) {
    throw new Error('User not found');
  }
  
  // Generate reset token
  const resetToken = crypto.randomBytes(32).toString('hex');
  const passwordResetToken = crypto
    .createHash('sha256')
    .update(resetToken)
    .digest('hex');
  
  // Set token expiration (1 hour)
  const passwordResetExpires = new Date(Date.now() + 3600000);
  
  // Save to user record
  await UserModel.findByIdAndUpdate(user._id, {
    passwordResetToken,
    passwordResetExpires
  });
  
  // Create reset URL
  const resetUrl = `${process.env.APP_URL}/reset-password/${resetToken}`;
  
  // Send email
  await sendEmail({
    to: user.email,
    subject: 'Password Reset Request',
    text: `To reset your password, please visit: ${resetUrl}`,
    html: `<p>To reset your password, please click <a href="${resetUrl}">here</a>.</p>`
  });
  
  return { message: 'Password reset email sent' };
}

async function resetPassword(token, newPassword) {
  // Hash the token
  const passwordResetToken = crypto
    .createHash('sha256')
    .update(token)
    .digest('hex');
  
  // Find user with valid token
  const user = await UserModel.findOne({
    passwordResetToken,
    passwordResetExpires: { $gt: Date.now() }
  });
  
  if (!user) {
    throw new Error('Invalid or expired reset token');
  }
  
  // Hash new password
  const saltRounds = 10;
  const passwordHash = await bcrypt.hash(newPassword, saltRounds);
  
  // Update user
  user.passwordHash = passwordHash;
  user.passwordResetToken = undefined;
  user.passwordResetExpires = undefined;
  await user.save();
  
  return { message: 'Password reset successful' };
}
```

## Session Management

### Session Handling

The application uses JWT tokens for session management:
- Access tokens with short expiry (15 minutes)
- Refresh tokens with longer expiry (7 days)
- Token revocation for logout

### Refresh Token Implementation

```javascript
// src/services/auth-service.js (continued)
async function refreshTokens(refreshToken) {
  if (!refreshToken) {
    throw new Error('Refresh token required');
  }
  
  try {
    // Verify refresh token
    const decoded = jwt.verify(refreshToken, process.env.JWT_REFRESH_SECRET);
    
    // Find user with matching refresh token
    const user = await UserModel.findOne({
      _id: decoded.userId,
      refreshToken
    });
    
    if (!user) {
      throw new Error('Invalid refresh token');
    }
    
    // Generate new tokens
    const newAccessToken = this.generateAccessToken(user);
    const newRefreshToken = this.generateRefreshToken(user);
    
    // Update refresh token in database
    await user.updateOne({ refreshToken: newRefreshToken });
    
    return {
      accessToken: newAccessToken,
      refreshToken: newRefreshToken
    };
  } catch (error) {
    throw new Error('Invalid or expired refresh token');
  }
}

async function logout(userId, refreshToken) {
  // Clear refresh token in database
  await UserModel.findByIdAndUpdate(userId, {
    refreshToken: null
  });
  
  return { message: 'Logout successful' };
}
```

## User Administration

### Admin Panel Features

The application includes an admin panel for user management:
- View all users
- Create new users
- Edit user details
- Assign roles
- Enable/disable user accounts
- Reset user passwords

### User Listing Example

```javascript
// src/services/admin-service.js
class AdminService {
  async listUsers(query = {}, options = {}) {
    const { skip = 0, limit = 20, sortBy = 'createdAt', sortOrder = -1 } = options;
    
    // Build query filters
    const filters = {};
    if (query.role) filters.role = query.role;
    if (query.isActive !== undefined) filters.isActive = query.isActive;
    if (query.search) {
      filters.$or = [
        { name: { $regex: query.search, $options: 'i' } },
        { email: { $regex: query.search, $options: 'i' } }
      ];
    }
    
    // Execute query with pagination
    const users = await UserModel.find(filters, {
      passwordHash: 0,
      refreshToken: 0
    })
      .sort({ [sortBy]: sortOrder })
      .skip(skip)
      .limit(limit)
      .lean();
    
    const total = await UserModel.countDocuments(filters);
    
    return {
      users,
      pagination: {
        total,
        page: Math.floor(skip / limit) + 1,
        pages: Math.ceil(total / limit),
        limit
      }
    };
  }
  
  // Other admin methods for user management
  // ...
}

module.exports = new AdminService();
```

## User Activity Logging

Track user actions for security and audit purposes:

```javascript
// src/services/activity-service.js
const ActivityModel = require('../models/activity-model');

class ActivityService {
  async logActivity(userId, action, details = {}) {
    const activity = new ActivityModel({
      userId,
      action,
      details,
      timestamp: new Date()
    });
    
    await activity.save();
    return activity;
  }
  
  async getUserActivities(userId, options = {}) {
    const { skip = 0, limit = 20 } = options;
    
    const activities = await ActivityModel.find({ userId })
      .sort({ timestamp: -1 })
      .skip(skip)
      .limit(limit);
      
    const total = await ActivityModel.countDocuments({ userId });
    
    return {
      activities,
      pagination: {
        total,
        page: Math.floor(skip / limit) + 1,
        pages: Math.ceil(total / limit),
        limit
      }
    };
  }
}

module.exports = new ActivityService();
```

## Best Practices

### Security Considerations

1. **Password Security**:
   - Enforce strong password policies
   - Hash passwords using bcrypt
   - Implement account lockout after failed attempts

2. **Authentication Best Practices**:
   - Use HTTPS for all communication
   - Implement proper token validation
   - Set secure and HTTP-only flags for cookies
   - Configure appropriate CORS settings

3. **Authorization Best Practices**:
   - Follow principle of least privilege
   - Validate permissions on both client and server side
   - Implement proper error handling for unauthorized access

### Performance Considerations

1. **Caching**:
   - Cache user permissions to reduce database queries
   - Cache user profiles for frequently accessed information

2. **Database Optimization**:
   - Create indexes for frequently queried fields
   - Implement pagination for user listings
   - Use projection to limit returned fields

## Related Documentation

- [Authentication Flow](../architecture/authentication-flow.md)
- [API Endpoints Reference](../api/endpoints-reference.md)
- [Security Best Practices Guide](security-best-practices-guide.md)
- [Development Environment Setup Guide](development-environment-setup.md)

## External Resources

- [OWASP Authentication Best Practices](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [JWT.io](https://jwt.io/)
- [MongoDB User Management](https://docs.mongodb.com/manual/administration/security-user-role-management/) 
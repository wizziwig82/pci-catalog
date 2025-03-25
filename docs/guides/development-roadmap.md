# Detailed Plan for Building a Music Catalog Management App

**Date**: [Current Date]

## Overview of the Application

You're building an application that will:
1. Upload and transcode audio files to Cloudflare R2 buckets
2. Manage metadata in MongoDB for these audio files
3. Allow users to edit metadata for both new and existing files
4. Enable replacement of audio files while preserving their metadata

Let me outline a comprehensive, step-by-step plan to build this application for someone with limited development experience.

## Phase 1: Planning and Setup

### Step 1: Requirements Gathering and Design (1-2 weeks)
- Define exact user requirements and workflows
- Create wireframes/mockups of the UI
- Plan the database schema (you already have a good start with your MongoDB schema)
- Plan the file storage structure in R2
- Design the API endpoints needed
- **Testing**: Review requirements with stakeholders to ensure all needs are captured

### Step 2: Development Environment Setup (2-3 days)
- Set up local development environment
- Install required tools: Node.js, MongoDB, code editor, Git
- Create a GitHub/GitLab repository
- Set up a project structure with proper folders
- Configure ESLint and Prettier for code formatting
- **Testing**: Verify all tools work correctly by creating and running basic scripts

### Step 3: Configure Access to External Services (1 week)
- Set up Cloudflare R2 account and create buckets
- Generate API keys for Cloudflare
- Set up MongoDB Atlas account (or local MongoDB)
- Create database and collections
- Set up secure credential management (environment variables)
- **Testing**: Test connections to both R2 and MongoDB with simple scripts

## Phase 2: Core Backend Implementation

### Step 4: Create Database Models (3-4 days)
- Implement MongoDB schema for tracks (and playlists if needed)
- Create models using Mongoose or similar ODM
- Implement data validation
- Set up indexes for performance
- **Testing**: Write tests for model creation, validation, and querying

### Step 5: Implement R2 Storage Integration (1 week)
- Set up Cloudflare R2 client
- Create functions for:
  - Uploading files to R2
  - Retrieving files from R2
  - Generating signed URLs for access
  - Deleting files from R2
- **Testing**: Test each R2 function independently with sample files

### Step 6: Implement Audio Processing (1-2 weeks)
- Research and select audio processing library (ffmpeg is recommended based on your requirements)
- Create a service for audio transcoding
- Implement functions to:
  - Extract metadata from audio files
  - Convert between formats
  - Generate waveforms/spectrograms if needed
  - Extract BPM and musical key if possible
- **Testing**: Test transcoding with various audio formats and verify quality

### Step 7: Create Core API Endpoints (1-2 weeks)
- Set up API framework (Express.js recommended)
- Implement endpoints for:
  - File upload
  - File retrieval
  - Metadata CRUD operations
  - Audio file replacement
- Add proper error handling and validation
- **Testing**: Test each endpoint with Postman or similar tool

## Phase 3: Frontend Development

### Step 8: Build Basic UI Structure (1 week)
- Set up frontend framework (React, Vue, or Angular)
- Create basic layout and navigation
- Implement authentication UI (if needed)
- **Testing**: Verify UI renders correctly across browsers

### Step 9: Implement File Upload Component (1 week)
- Create drag-and-drop upload zone
- Add progress indicators
- Implement file validation
- Add cancel functionality
- **Testing**: Test uploads with various file sizes and types

### Step 10: Create Metadata Editor (1-2 weeks)
- Build form for track metadata
- Add validation for required fields
- Implement auto-extraction of metadata from files
- Create preview functionality for audio and images
- **Testing**: Test form submission with various inputs

### Step 11: Implement Track Management UI (1-2 weeks)
- Create listing/search interface for existing tracks
- Build detailed view for individual tracks
- Add editing capabilities
- Implement file replacement functionality
- **Testing**: Test all UI interactions and CRUD operations

## Phase 4: Integration and Advanced Features

### Step 12: Implement Authentication and Authorization (1 week)
- Set up authentication system
- Create user roles and permissions
- Secure API endpoints
- Implement session management
- **Testing**: Test login, logout, and access restrictions

### Step 13: Add Batch Operations (1 week)
- Implement bulk upload
- Add batch editing capabilities
- Create export functionality
- **Testing**: Test with large batches of files and metadata

### Step 14: Implement Search and Filtering (1 week)
- Create advanced search functionality
- Add filtering by various metadata
- Implement sorting options
- **Testing**: Test search performance with large datasets

### Step 15: Add Analytics and Reporting (1 week)
- Implement usage statistics
- Create storage utilization monitoring
- Add activity logging
- **Testing**: Verify accuracy of analytics data

## Phase 5: Testing and Deployment

### Step 16: Comprehensive Testing (2 weeks)
- Conduct unit testing for all components
- Perform integration testing
- Execute end-to-end testing
- Test performance with large datasets
- Conduct security testing
- **Testing**: Document and fix all issues found

### Step 17: Optimization (1 week)
- Improve performance bottlenecks
- Optimize database queries
- Enhance file processing speed
- **Testing**: Measure performance improvements

### Step 18: Documentation (1 week)
- Create user documentation
- Write technical documentation
- Document API endpoints
- **Testing**: Have someone follow documentation to verify clarity

### Step 19: Deployment Setup (1 week)
- Configure production environment
- Set up CI/CD pipeline
- Implement logging and monitoring
- Configure backup systems
- **Testing**: Test deployment process in staging environment

### Step 20: Launch (ongoing)
- Deploy to production
- Monitor for issues
- Gather user feedback
- Plan for future improvements
- **Testing**: Conduct post-launch testing

## Key Technical Components

### 1. Backend Framework
- **Node.js with Express.js**: Provides a robust framework for building the API
- **MongoDB**: For storing metadata about tracks
- **Mongoose**: ODM for MongoDB to simplify database operations

### 2. File Storage and Processing
- **Cloudflare R2**: For storing audio files
- **FFmpeg**: For audio transcoding and metadata extraction
- **music-metadata** or similar: For parsing and extracting audio metadata

### 3. Frontend
- **React/Vue.js**: For building the UI
- **Axios/fetch**: For API communication
- **React Dropzone/Vue Upload Component**: For file uploads
- **Form libraries**: For metadata editing

### 4. Authentication/Authorization
- **JWT**: For secure authentication
- **Passport.js**: For authentication strategies
- **bcrypt**: For password hashing

## Important Considerations

### Performance
- Implement chunked uploads for large files
- Consider worker threads for transcoding
- Use MongoDB indexes effectively
- Implement caching where appropriate

### Security
- Secure all API endpoints
- Validate all user inputs
- Use HTTPS for all communications
- Implement proper authentication and authorization
- Set appropriate CORS policies

### Scalability
- Design with horizontal scaling in mind
- Consider implementing a queue for transcoding jobs
- Use database connection pooling

## Specific Implementation Guidance for Coding with Claude

When implementing this project using Claude Sonnet in an agentic program like Cursor, follow these guidelines:

1. **Break down tasks into small, focused prompts**:
   - Ask for one component or feature at a time
   - Be specific about what you need in each prompt

2. **Build incrementally**:
   - Start with core functionality before adding advanced features
   - Test each component thoroughly before moving on

3. **For complex code generation**:
   - Provide Claude with context about what's already implemented
   - Explain the specific problem you're trying to solve
   - Include any error messages or issues you're encountering

4. **Example prompts structure**:
   - "Create a MongoDB schema for tracks based on the requirements"
   - "Implement a function to upload files to Cloudflare R2"
   - "Build an Express route for updating track metadata"
   - "Create a React component for the file upload interface"

5. **When requesting code**:
   - Specify the programming language and framework
   - Include any dependencies or imports needed
   - Mention error handling requirements
   - Ask for comments explaining complex parts

By following this detailed plan and approach to implementing with Claude, you should be able to successfully build your music catalog management application in a structured, efficient manner while minimizing potential issues along the way. 
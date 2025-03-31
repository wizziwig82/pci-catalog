# MongoDB Setup for Music Library Manager

This document outlines the MongoDB setup process and database structure for the Music Library Manager application.

## Database Details

- **Database Name**: `music_library`
- **Collections**: 
  - `albums` - Stores album information
  - `tracks` - Stores track information

## Setup Instructions

### Prerequisites

- MongoDB installed and running (version 4.4+ recommended)
  ```bash
  # Install MongoDB on macOS with Homebrew
  brew tap mongodb/brew
  brew install mongodb-community
  
  # Start MongoDB service
  brew services start mongodb/brew/mongodb-community
  ```
- Node.js (for running setup scripts)
- MongoDB Node.js driver
  ```bash
  npm install mongodb
  ```

### Setting Up the Database

1. **Using ES Modules (Recommended for Node.js)**
   ```bash
   # Run the setup script
   node scripts/setup_mongodb.mjs
   
   # Test the connection
   node scripts/test_mongodb_connection.mjs
   ```

2. **Using the MongoDB Shell**
   ```bash
   # Connect to MongoDB and run the setup script
   mongosh mongodb://localhost:27017/music_library scripts/mongodb_setup.js
   ```

### Verifying the Setup

Once the setup is complete, you can run the test script to verify that the database is working correctly:

```bash
node scripts/test_mongodb_connection.mjs
```

This will:
1. Connect to MongoDB
2. List the collections
3. Create a test album and track
4. Test querying and relationships
5. Test text search
6. Clean up test data

## Database Schema

### Albums Collection

```json
{
  "_id": ObjectId,
  "name": String,         // required
  "track_ids": [ObjectId], // required, references to tracks
  "art_path": String,     // optional, path to album artwork
  "release_date": Date,   // optional
  "publisher": String     // optional
}
```

### Tracks Collection

```json
{
  "_id": ObjectId,
  "title": String,        // required
  "album_id": ObjectId,   // required, reference to parent album
  "track_number": Int,    // optional
  "filename": String,     // required, original filename
  "duration": Int,        // required, in seconds
  "writers": [String],    // required
  "publishers": [String], // required
  "composers": [String],  // optional
  "genre": String,        // optional
  "path": String,         // required, file path in storage
  "waveform_data": [Int]  // optional, for visualization
}
```

## Indexes

The database setup creates the following indexes for efficient operations:

1. **Text index on album names** - For searching albums by name
   ```
   db.albums.createIndex({ name: "text" })
   ```

2. **Text index on track titles and genres** - For searching tracks
   ```
   db.tracks.createIndex({ title: "text", genre: "text" })
   ```

3. **Index on album_id in tracks** - For efficient lookup of tracks by album
   ```
   db.tracks.createIndex({ album_id: 1 })
   ```

## Common Queries

### Finding an Album with All Its Tracks

```javascript
const album = await db.collection('albums').findOne({ _id: albumId });
const tracks = await db.collection('tracks').find({ album_id: albumId }).toArray();
```

### Searching for Albums

```javascript
const searchResults = await db.collection('albums').find(
  { $text: { $search: "search term" } }
).toArray();
```

### Searching for Tracks

```javascript
const searchResults = await db.collection('tracks').find(
  { $text: { $search: "search term" } }
).toArray();
```

## Environment Configuration

The application uses the following environment variables for MongoDB configuration:

- `MONGODB_URI` - MongoDB connection string (default: `mongodb://localhost:27017`)
- `MONGODB_DB_NAME` - Database name (default: `music_library`)

These can be set in your `.env` file or passed as environment variables when running the application.

## Troubleshooting

If you encounter issues with the MongoDB connection:

1. **Check MongoDB service**
   ```bash
   # Check if MongoDB is running
   brew services list | grep mongodb
   
   # Restart MongoDB if needed
   brew services restart mongodb/brew/mongodb-community
   ```

2. **Check MongoDB connection**
   ```bash
   # Connect using mongosh
   mongosh mongodb://localhost:27017
   ```

3. **Error: "Failed to load resource: Could not connect to the server"**
   - Make sure MongoDB is installed and running
   - Check your connection string in the application
   - Verify that the MongoDB service is active

4. **Schema validation errors**
   - Ensure your data matches the required schema structure
   - Check that required fields are present in your documents
   - Verify field types match the schema requirements 
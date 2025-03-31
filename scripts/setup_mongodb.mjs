// MongoDB Setup Script for Music Library Manager (ES Module version)
// Run with:
// node scripts/setup_mongodb.mjs

import { MongoClient } from 'mongodb';

// Update these connection details as needed
const uri = process.env.MONGODB_URI || 'mongodb://localhost:27017';
const dbName = 'music_library';

async function setupDatabase() {
  console.log(`Connecting to MongoDB at ${uri}...`);
  const client = new MongoClient(uri);
  
  try {
    await client.connect();
    console.log('Connected successfully');
    
    const db = client.db(dbName);
    console.log(`Using database: ${dbName}`);
    
    await createCollections(db);
    console.log('Database setup completed successfully');
  } catch (err) {
    console.error('Error setting up database:', err);
  } finally {
    await client.close();
    console.log('MongoDB connection closed');
  }
}

async function createCollections(db) {
  console.log('Creating collections and indexes...');
  
  // Create albums collection with schema validation
  try {
    await db.createCollection('albums', {
      validator: {
        $jsonSchema: {
          bsonType: 'object',
          required: ['name', 'track_ids'],
          properties: {
            name: {
              bsonType: 'string',
              description: 'Album name is required'
            },
            track_ids: {
              bsonType: 'array',
              description: 'Array of track IDs in this album',
              items: {
                bsonType: 'objectId'
              }
            },
            art_path: {
              bsonType: 'string',
              description: 'Path to album artwork'
            },
            release_date: {
              bsonType: 'date',
              description: 'Album release date'
            },
            publisher: {
              bsonType: 'string',
              description: 'Album publisher'
            }
          }
        }
      }
    });
    console.log('Created albums collection');
  } catch (err) {
    // Collection might already exist
    console.log('Albums collection may already exist:', err.message);
  }
  
  // Create tracks collection with schema validation
  try {
    await db.createCollection('tracks', {
      validator: {
        $jsonSchema: {
          bsonType: 'object',
          required: ['title', 'album_id', 'filename', 'duration', 'writers', 'publishers', 'path'],
          properties: {
            title: {
              bsonType: 'string',
              description: 'Track title is required'
            },
            album_id: {
              bsonType: 'objectId',
              description: 'Reference to album'
            },
            track_number: {
              bsonType: 'int',
              description: 'Track number on album'
            },
            filename: {
              bsonType: 'string',
              description: 'Original filename'
            },
            duration: {
              bsonType: 'int',
              description: 'Track duration in seconds'
            },
            writers: {
              bsonType: 'array',
              description: 'List of writers',
              items: {
                bsonType: 'string'
              }
            },
            publishers: {
              bsonType: 'array',
              description: 'List of publishers',
              items: {
                bsonType: 'string'
              }
            },
            composers: {
              bsonType: 'array',
              description: 'List of composers',
              items: {
                bsonType: 'string'
              }
            },
            genre: {
              bsonType: 'string',
              description: 'Music genre'
            },
            path: {
              bsonType: 'string',
              description: 'File path in storage'
            },
            waveform_data: {
              bsonType: 'array',
              description: 'Waveform visualization data',
              items: {
                bsonType: 'int'
              }
            }
          }
        }
      }
    });
    console.log('Created tracks collection');
  } catch (err) {
    // Collection might already exist
    console.log('Tracks collection may already exist:', err.message);
  }
  
  // Create indexes for efficient searching and relationships
  
  // Text index for album search
  await db.collection('albums').createIndex(
    { name: "text" },
    { name: "album_name_text" }
  );
  console.log('Created album_name_text index');
  
  // Text index for track search
  await db.collection('tracks').createIndex(
    { title: "text", genre: "text" },
    { name: "track_search_text" }
  );
  console.log('Created track_search_text index');
  
  // Index for album_id to establish relationship
  await db.collection('tracks').createIndex(
    { album_id: 1 },
    { name: "album_track_relationship" }
  );
  console.log('Created album_track_relationship index');
}

// Run the setup
setupDatabase(); 
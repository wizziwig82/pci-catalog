// MongoDB Setup Script for Music Library Manager
// Run this script using mongo shell:
// mongo mongodb://localhost:27017/music_library scripts/mongodb_setup.js
//
// Or with Node.js:
// node scripts/mongodb_setup.js

// We recommend using setup_mongodb.mjs for Node.js environments
// This file is primarily for MongoDB shell usage

// When running in mongo shell, db is already defined
if (typeof db !== 'undefined') {
  createCollections(db);
  print('Database setup completed successfully');
}

function createCollections(db) {
  // Use MongoDB shell syntax or Node.js MongoDB driver as appropriate
  
  // Create albums collection with schema validation
  db.createCollection('albums', {
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
  
  // Create tracks collection with schema validation
  db.createCollection('tracks', {
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
  
  // Create indexes for efficient searching and relationships
  
  // Text index for album search
  db.albums.createIndex(
    { name: "text" },
    { name: "album_name_text" }
  );
  
  // Text index for track search
  db.tracks.createIndex(
    { title: "text", genre: "text" },
    { name: "track_search_text" }
  );
  
  // Index for album_id to establish relationship
  db.tracks.createIndex(
    { album_id: 1 },
    { name: "album_track_relationship" }
  );
  
  // Output created collections and indexes (works in both MongoDB shell and Node.js)
  if (typeof console !== 'undefined') {
    console.log('Created collections:');
    console.log('- albums');
    console.log('- tracks');
    console.log('Created indexes:');
    console.log('- album_name_text');
    console.log('- track_search_text');
    console.log('- album_track_relationship');
  } else {
    print('Created collections:');
    print('- albums');
    print('- tracks');
    print('Created indexes:');
    print('- album_name_text');
    print('- track_search_text');
    print('- album_track_relationship');
  }
} 
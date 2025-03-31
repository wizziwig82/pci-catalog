// MongoDB Setup Script for Music Library Manager
//
// This script creates the necessary collections and indexes for the Music Library Manager application.
// It can be run using the MongoDB shell or with Node.js and the MongoDB driver.
//
// To run with MongoDB shell:
// mongosh "mongodb://localhost:27017/music_library" setup_mongodb.js
//
// To run with Node.js:
// node setup_mongodb.js

// Database setup - create or use music_library database
db = db.getSiblingDB('music_library');

// Create albums collection with validation
db.createCollection('albums', {
  validator: {
    $jsonSchema: {
      bsonType: 'object',
      required: ['name', 'track_ids'],
      properties: {
        name: {
          bsonType: 'string',
          description: 'Album name'
        },
        art_path: {
          bsonType: ['string', 'null'],
          description: 'Path to album artwork in R2 (optional)'
        },
        track_ids: {
          bsonType: 'array',
          description: 'Array of track IDs associated with this album',
          items: {
            bsonType: 'string'
          }
        }
      }
    }
  }
});

// Create tracks collection with validation
db.createCollection('tracks', {
  validator: {
    $jsonSchema: {
      bsonType: 'object',
      required: ['title', 'album_id', 'filename', 'duration', 'writers', 'publishers', 'path'],
      properties: {
        title: {
          bsonType: 'string',
          description: 'Track title'
        },
        album_id: {
          bsonType: 'string',
          description: 'Reference to parent album'
        },
        filename: {
          bsonType: 'string',
          description: 'Original filename'
        },
        duration: {
          bsonType: 'double',
          description: 'Track duration in seconds'
        },
        comments: {
          bsonType: ['string', 'null'],
          description: 'Additional track comments (optional)'
        },
        writers: {
          bsonType: 'array',
          description: 'Array of writers with percentage ownership',
          items: {
            bsonType: 'object',
            required: ['name', 'percentage'],
            properties: {
              name: {
                bsonType: 'string',
                description: "Writer's name"
              },
              percentage: {
                bsonType: 'double',
                description: 'Percentage ownership'
              }
            }
          }
        },
        publishers: {
          bsonType: 'array',
          description: 'Array of publishers with percentage ownership',
          items: {
            bsonType: 'object',
            required: ['name', 'percentage'],
            properties: {
              name: {
                bsonType: 'string',
                description: "Publisher's name"
              },
              percentage: {
                bsonType: 'double',
                description: 'Percentage ownership'
              }
            }
          }
        },
        genre: {
          bsonType: ['string', 'null'],
          description: 'Music genre (optional)'
        },
        instruments: {
          bsonType: 'array',
          description: 'Array of instruments used in the track',
          items: {
            bsonType: 'string'
          }
        },
        mood: {
          bsonType: ['string', 'null'],
          description: 'Track mood descriptor (optional)'
        },
        path: {
          bsonType: 'object',
          required: ['original', 'medium', 'low'],
          description: 'Paths to different quality versions in R2',
          properties: {
            original: {
              bsonType: 'string',
              description: 'Path to original quality file'
            },
            medium: {
              bsonType: 'string',
              description: 'Path to medium quality file'
            },
            low: {
              bsonType: 'string',
              description: 'Path to low quality file'
            }
          }
        }
      }
    }
  }
});

// Create indexes

// Text indexes for search functionality
db.albums.createIndex({ name: 'text' });
db.tracks.createIndex({ title: 'text', genre: 'text' });

// Regular index for album-track relationship
db.tracks.createIndex({ album_id: 1 });

print('MongoDB setup for Music Library Manager completed successfully.');
print('Created collections: albums, tracks');
print('Created indexes for efficient searching and relationship lookups.'); 
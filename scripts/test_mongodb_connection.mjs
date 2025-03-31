// MongoDB Connection Test Script (ES Module version)
// Run with:
// node scripts/test_mongodb_connection.mjs

import { MongoClient, ObjectId } from 'mongodb';

// Update these connection details as needed
const uri = process.env.MONGODB_URI || 'mongodb://localhost:27017';
const dbName = 'music_library';

async function testConnection() {
  console.log('Testing MongoDB connection...');
  const client = new MongoClient(uri);
  
  try {
    // Connect to the MongoDB server
    await client.connect();
    console.log('Successfully connected to MongoDB');
    
    // Access the database
    const db = client.db(dbName);
    console.log(`Connected to database: ${dbName}`);
    
    // List all collections
    const collections = await db.listCollections().toArray();
    console.log('\nCollections:');
    collections.forEach(collection => {
      console.log(`- ${collection.name}`);
    });
    
    // Test inserting and retrieving test data
    console.log('\nTesting basic operations...');
    
    // Create a test album
    const albumsCollection = db.collection('albums');
    const testAlbum = {
      name: 'Test Album',
      track_ids: [],
      art_path: '/path/to/test/album/art.jpg',
      release_date: new Date('2023-01-01')
    };
    
    const albumResult = await albumsCollection.insertOne(testAlbum);
    console.log(`Inserted test album with ID: ${albumResult.insertedId}`);
    
    // Create a test track
    const tracksCollection = db.collection('tracks');
    const testTrack = {
      title: 'Test Track',
      album_id: albumResult.insertedId,
      track_number: 1,
      filename: 'test_track.mp3',
      duration: 180,
      writers: ['Test Writer'],
      publishers: ['Test Publisher'],
      composers: ['Test Composer'],
      genre: 'Test Genre',
      path: '/path/to/test/track.mp3',
      waveform_data: [1, 2, 3, 4, 5]
    };
    
    const trackResult = await tracksCollection.insertOne(testTrack);
    console.log(`Inserted test track with ID: ${trackResult.insertedId}`);
    
    // Update album with track ID
    await albumsCollection.updateOne(
      { _id: albumResult.insertedId },
      { $push: { track_ids: trackResult.insertedId } }
    );
    console.log('Updated album with track reference');
    
    // Retrieve album with populated tracks
    const populatedAlbum = await albumsCollection.findOne({ _id: albumResult.insertedId });
    console.log('\nRetrieved album:');
    console.log(populatedAlbum);
    
    // Retrieve track by album relationship
    const relatedTracks = await tracksCollection.find({ album_id: albumResult.insertedId }).toArray();
    console.log('\nRelated tracks:');
    console.log(relatedTracks);
    
    // Test text search
    console.log('\nTesting text search...');
    const searchResults = await tracksCollection.find(
      { $text: { $search: 'Test' } }
    ).toArray();
    console.log(`Found ${searchResults.length} tracks matching 'Test'`);
    
    // Clean up test data
    console.log('\nCleaning up test data...');
    await tracksCollection.deleteOne({ _id: trackResult.insertedId });
    await albumsCollection.deleteOne({ _id: albumResult.insertedId });
    console.log('Test data cleaned up');
    
    console.log('\nAll tests completed successfully!');
  } catch (err) {
    console.error('Error testing MongoDB connection:', err);
  } finally {
    await client.close();
    console.log('MongoDB connection closed');
  }
}

testConnection(); 
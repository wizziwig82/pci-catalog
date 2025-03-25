// Test script to verify MongoDB Atlas connection
require('dotenv').config();
const mongoose = require('mongoose');

// Get MongoDB URI from .env file
const uri = process.env.MONGODB_URI;

if (!uri) {
  console.error('MongoDB URI is not defined in .env file');
  process.exit(1);
}

console.log('Attempting to connect to MongoDB...');
console.log(`URI: ${uri.replace(/mongodb\+srv:\/\/([^:]+):([^@]+)@/, 'mongodb+srv://******:******@')}`);

// Connect to MongoDB with updated options for newer MongoDB versions
mongoose.connect(uri, {
  // Use the new API rather than deprecated options
  serverSelectionTimeoutMS: 5000, // Timeout after 5s instead of 30s
})
  .then(() => {
    console.log('✅ Successfully connected to MongoDB Atlas!');
    console.log('Database connection is working correctly.');
    
    // Close the connection after successful test
    return mongoose.connection.close();
  })
  .then(() => {
    console.log('Connection closed.');
    process.exit(0);
  })
  .catch(err => {
    console.error('❌ Error connecting to MongoDB:', err.message);
    
    if (err.message.includes('bad auth') || err.message.includes('Authentication failed')) {
      console.log('\nAuthentication Error:');
      console.log('1. Double-check your username and password');
      console.log('2. Make sure special characters in password are properly URL encoded');
      console.log('3. Verify the user has the correct access permissions');
      console.log('4. Ensure you\'ve whitelisted your current IP address in MongoDB Atlas');
    } else if (err.name === 'MongoServerSelectionError') {
      console.log('\nConnection Error:');
      console.log('1. Check your network connection');
      console.log('2. Verify the cluster address is correct');
      console.log('3. Ensure your IP address is whitelisted in MongoDB Atlas');
    }
    
    process.exit(1);
  }); 
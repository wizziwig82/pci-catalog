#!/usr/bin/env node
require('dotenv').config();
const { execSync } = require('child_process');
const path = require('path');
const username = process.env.GITHUB_USERNAME;
const token = process.env.GITHUB_TOKEN;

// Add debugging
console.log('Username:', username);
console.log('Token length:', token ? token.length : 0);
console.log('First few chars of token:', token ? token.substring(0, 4) : 'none');

const branch = process.argv[2] || 'main';
const commitMessage = process.argv[3] || 'Update from script';
const repoUrlWithAuth = `https://${username}:${token}@github.com/${username}/pci-catalog.git`;

// Debug the URL (but mask most of the token)
const maskedUrl = `https://${username}:${token ? token.substring(0, 4) + '...' : 'none'}@github.com/${username}/pci-catalog.git`;
console.log('Using repo URL:', maskedUrl);

try {
  execSync('git add .', { stdio: 'inherit' });
  execSync(`git commit -m "${commitMessage}"`, { stdio: 'inherit' });
  execSync(`git remote set-url origin ${repoUrlWithAuth}`);
  execSync(`git push -u origin ${branch}`, { stdio: 'inherit' });
  console.log('Push completed successfully!');
} catch (error) {
  console.error('Error executing Git commands:', error.message);
  process.exit(1);
}

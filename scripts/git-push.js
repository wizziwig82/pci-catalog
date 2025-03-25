#!/usr/bin/env node
require('dotenv').config();
const { execSync } = require('child_process');
const path = require('path');
const username = process.env.GITHUB_USERNAME;

// Add debugging
console.log('Username:', username);
console.log('Using SSH authentication');

const branch = process.argv[2] || 'main';
const commitMessage = process.argv[3] || 'Update from script';

// Use SSH URL instead of HTTPS with token
const repoUrl = `git@github.com:wizziwig82/pci-catalog.git`;
console.log('Using repo URL:', repoUrl);

try {
  execSync('git add .', { stdio: 'inherit' });
  execSync(`git commit -m "${commitMessage}"`, { stdio: 'inherit' });
  execSync(`git remote set-url origin ${repoUrl}`);
  execSync(`git push -u origin ${branch}`, { stdio: 'inherit' });
  console.log('Push completed successfully!');
} catch (error) {
  console.error('Error executing Git commands:', error.message);
  process.exit(1);
}

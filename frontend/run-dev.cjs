#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

// Get npm executable path
const npmCmd = process.platform === 'win32' ? 'npm.cmd' : 'npm';

// Spawn npm run dev
const child = spawn(npmCmd, ['run', 'dev'], {
  cwd: __dirname,
  stdio: 'inherit',
  shell: true
});

child.on('error', (error) => {
  console.error('Failed to start frontend:', error);
  process.exit(1);
});

child.on('exit', (code) => {
  process.exit(code);
});

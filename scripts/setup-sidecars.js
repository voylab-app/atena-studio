#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const isWindows = process.platform === 'win32';
const scriptDir = __dirname;

let command;
let args;

if (isWindows) {
  const psScript = path.join(scriptDir, 'setup-sidecars.ps1');
  // Use powershell.exe which is available on all Windows systems by default
  command = 'powershell.exe';
  args = ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', psScript];
} else {
  const shScript = path.join(scriptDir, 'setup-sidecars.sh');
  try {
    fs.chmodSync(shScript, 0o755);
  } catch (_) {}
  command = 'bash';
  args = [shScript];
}

const child = spawn(command, args, {
  stdio: 'inherit',
  shell: isWindows,
  env: process.env,
});

child.on('error', (err) => {
  console.error(`Failed to execute setup sidecars script (${command}):`, err);
  process.exit(1);
});

child.on('exit', (code, signal) => {
  if (signal) {
    process.exit(1);
  }
  process.exit(code ?? 0);
});

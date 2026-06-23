#!/usr/bin/env node
/**
 * Workspace test runner with Windows-safe linker concurrency.
 * MSVC link.exe can fail with LNK1104 when multiple test binaries link in parallel.
 */
import { spawnSync } from 'node:child_process';

const cargoArgs = ['test', '--workspace'];
if (process.platform === 'win32') {
  cargoArgs.push('-j', '1');
}

const result = spawnSync('cargo', cargoArgs, {
  stdio: 'inherit',
  shell: process.platform === 'win32',
});

process.exit(result.status ?? 1);

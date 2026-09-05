#!/usr/bin/env node

const { spawnSync } = require('child_process');

const { ensureBinary } = require('../lib/download');
const { getBinaryPath } = require('../lib/config');

async function run() {
  let binaryPath = getBinaryPath();

  if (!binaryPath) {
    try {
      binaryPath = await ensureBinary();
    } catch (err) {
      console.error(`keyrate: ${err.message}`);
      console.error(`keyrate: check your network connection and try again.`);
      process.exit(1);
    }
  }

  const args = process.argv.slice(2);
  const result = spawnSync(binaryPath, args, {
    stdio: 'inherit',
    shell: process.platform === 'win32',
    env: process.env,
  });

  if (result.error) {
    console.error(`keyrate: failed to run binary: ${result.error.message}`);
    process.exit(1);
  }

  process.exit(result.status ?? 0);
}

run();

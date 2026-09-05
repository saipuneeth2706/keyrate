const fs = require('fs');
const https = require('https');
const os = require('os');
const path = require('path');

const { BASE_URL, LATEST_BASE_URL, getPlatformInfo, getVersion, getCacheDir } = require('./config');

function download(url, dest) {
  return new Promise((resolve, reject) => {
    fs.mkdirSync(path.dirname(dest), { recursive: true });
    const file = fs.createWriteStream(dest);
    const req = https.get(url, (response) => {
      if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
        file.close();
        req.destroy();
        return download(response.headers.location, dest).then(resolve, reject);
      }
      if (response.statusCode !== 200) {
        file.close();
        reject(new Error(`HTTP ${response.statusCode}`));
        return;
      }
      response.pipe(file);
      file.on('finish', () => {
        file.close();
        resolve();
      });
    });
    req.on('error', (err) => {
      try { fs.unlinkSync(dest); } catch (_) {}
      reject(err);
    });
    file.on('error', (err) => {
      try { fs.unlinkSync(dest); } catch (_) {}
      reject(err);
    });
  });
}

async function ensureBinary() {
  const { platform, assetName, binaryName } = getPlatformInfo();
  const cacheDir = getCacheDir();
  const version = getVersion() || 'latest';
  const dir = path.join(cacheDir, version);
  const binaryPath = path.join(dir, binaryName);

  if (fs.existsSync(binaryPath)) {
    return binaryPath;
  }

  const urls = [
    `${BASE_URL}/${version}/${assetName}`,
    `${LATEST_BASE_URL}/${assetName}`,
  ];

  const tmp = path.join(dir, `.${binaryName}.${process.pid}.tmp`);
  for (const url of urls) {
    console.log(`keyrate: downloading ${url}`);
    try {
      await download(url, tmp);
      if (platform !== 'win32') fs.chmodSync(tmp, 0o755);
      fs.renameSync(tmp, binaryPath);
      console.log(`keyrate: installed ${binaryName} to ${binaryPath}`);
      return binaryPath;
    } catch (err) {
      console.log(`keyrate: failed (${err.message}), trying next source`);
    }
  }

  try { fs.unlinkSync(tmp); } catch (_) {}
  throw new Error(
    `could not download the binary. Make sure a GitHub Release exists (tag ${version}) ` +
    `for ${require('./config').GITHUB_OWNER}/${require('./config').GITHUB_REPO}.`
  );
}

module.exports = { ensureBinary };

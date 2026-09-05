const { getBinaryPath } = require('./lib/config');
const { ensureBinary } = require('./lib/download');

module.exports = {
  getBinaryPath,
  async ensureBinary() {
    return ensureBinary();
  },
};

// node-runner.js
const http = require('http');
const vm = require('vm');
const crypto = require('crypto');

let state0, state1;

function initializeRandomGenerator(entropyHex) {
  console.log('[initializeRandomGenerator] Received entropy hex:', entropyHex);

  const hostBytes = Buffer.from(entropyHex || crypto.randomBytes(16).toString('hex'), 'hex');
  const seed = crypto.createHash('sha256').update(hostBytes).digest();

  state0 = seed.readBigUInt64BE(0);
  state1 = seed.readBigUInt64BE(8);

  for (let i = 0; i < 20; i++) xoroshiro128plus();

  console.log('[initializeRandomGenerator] Math.random has been reseeded.');
}

function xoroshiro128plus() {
  const result = state0 + state1;
  state1 ^= state0;
  state0 = rotl(state0, 55n) ^ state1 ^ (state1 << 14n);
  state1 = rotl(state1, 36n);
  return result;
}

function rotl(x, k) {
  return (x << k) | (x >> (64n - k));
}

function customRandom() {
  return Number(xoroshiro128plus() & BigInt(0x1fffffffffffff)) / 0x200000000000;
}

// Initialize with some default entropy
initializeRandomGenerator();

const server = http.createServer(async (req, res) => {
  if (req.method === 'POST' && req.url === '/run') {
    let body = [];
    req.on('data', chunk => body.push(chunk));
    req.on('end', () => {
      body = Buffer.concat(body).toString();
      let jsonBody;
      try {
        jsonBody = JSON.parse(body);
      } catch (e) {
        res.writeHead(400, { 'Content-Type': 'application/json' });
        return res.end(JSON.stringify({ error: 'Invalid JSON body' }));
      }

      const { code, entropy } = jsonBody;

      // Reset randomness if entropy is provided
      if (entropy) {
        initializeRandomGenerator(entropy);
      }

      let stdout = '';
      let stderr = '';
      let exitCode = 0;

      try {
        const sandboxConsole = {
          log: (...args) => { stdout += args.join(' ') + '\n'; },
          error: (...args) => { stderr += args.join(' ') + '\n'; }
        };

        const sandbox = {
          console: sandboxConsole,
          result: null,
          crypto: crypto,
          Math: { ...Math, random: customRandom }
        };

        vm.runInNewContext(code, sandbox, { timeout: 360000 });
      } catch (err) {
        stderr += (err.stack || err.toString()) + '\n';
        exitCode = 1;
      }

      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ stdout, stderr, exit_code: exitCode }));
    });
  } else {
    res.writeHead(404, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'Not found' }));
  }
});

server.listen(5000, () => {
  console.log('node-runner listening on port 5000');
});

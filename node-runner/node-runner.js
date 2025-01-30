// node-runner.js
const http = require('http');
const vm = require('vm');
const crypto = require('crypto');
const seedrandom = require('seedrandom');

// In case we also want to keep track of the latest entropy
let globalEntropy = Buffer.alloc(32);

// This function uses the hex string `entropy` from the host
// to either set or mix your Node app's random seed. It also
// monkey‐patches `Math.random()` to use a seed from 'seedrandom'.
function resetRandomness(entropyHex) {
  console.log('[resetRandomness] Received entropy hex:', entropyHex);

  // 1) Convert the hex string into raw bytes
  let hostBytes = Buffer.from(entropyHex, 'hex');

  // OPTIONAL: if you want to combine host entropy with some local randomness,
  // you could do something like:
  // crypto.randomFillSync(hostBytes);

  // 2) Store or track it globally, if you like
  globalEntropy = hostBytes;

  // 3) Create a deterministic seed (string) from those bytes
  //    Here we use a SHA-256 hash of the raw bytes to get a uniform 64-hex-char string.
  const seed = crypto.createHash('sha256')
    .update(hostBytes)
    .digest('hex');

  // 4) Monkey‐patch Math.random() using seedrandom
  //    This replaces the system’s (stale) PRNG with a seeded PRNG.
  //    On each snapshot restore + /reset_entropy call, you'll get a new seed.
  Math.random = seedrandom(seed);

  console.log('[resetRandomness] Math.random has been reseeded.');
}

const server = http.createServer(async (req, res) => {
  // 1) /reset_entropy endpoint: POST JSON { "entropy": "deadbeef..." }
  if (req.method === 'POST' && req.url === '/reset_entropy') {
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

      const { entropy } = jsonBody;
      // Basic validation: must be a hex string of even length, etc.
      if (!entropy || typeof entropy !== 'string' || entropy.length % 2 !== 0) {
        res.writeHead(400, { 'Content-Type': 'application/json' });
        return res.end(JSON.stringify({ error: 'Invalid entropy provided' }));
      }

      try {
        resetRandomness(entropy);
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ message: 'Entropy reset successfully' }));
      } catch (err) {
        console.error('[reset_entropy] Error:', err);
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: 'Failed to reset entropy', details: err.toString() }));
      }
    });

  // 2) /run endpoint: run code in a VM sandbox
  } else if (req.method === 'POST' && req.url === '/run') {
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

      const code = jsonBody.code ?? '';

      let stdout = '';
      let stderr = '';
      let exitCode = 0;

      try {
        // A custom console for capturing logs from the sandbox
        const sandboxConsole = {
          log: (...args) => { stdout += args.join(' ') + '\n'; },
          error: (...args) => { stderr += args.join(' ') + '\n'; }
        };

        // Expose a few globals in the sandbox:
        const sandbox = {
          console: sandboxConsole,
          result: null,
          crypto: crypto,
          // Ensure the sandbox can also see the monkey-patched Math:
          Math: global.Math
        };

        // Safely run user code with a timeout
        vm.runInNewContext(code, sandbox, { timeout: 360000 });
      } catch (err) {
        stderr += (err.stack || err.toString()) + '\n';
        exitCode = 1;
      }

      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ stdout, stderr, exit_code: exitCode }));
    });
  } else {
    // 404 for any other paths
    res.writeHead(404, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'Not found' }));
  }
});

server.listen(5000, () => {
  console.log('node-runner listening on port 5000');
});


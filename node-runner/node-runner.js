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
    req.on('end', async () => {
      body = Buffer.concat(body).toString();
      let jsonBody;
      try {
        jsonBody = JSON.parse(body);
      } catch (e) {
        res.writeHead(400, { 'Content-Type': 'application/json' });
        return res.end(JSON.stringify({ error: 'Invalid JSON body' }));
      }

      const { code, entropy } = jsonBody;

      // Optionally re-seed your custom RNG
      if (entropy) {
        initializeRandomGenerator(entropy);
      }

      let stdout = '';
      let stderr = '';
      let exitCode = 0;

      // Save original console methods
      const originalLog = console.log;
      const originalError = console.error;

      // Override console to capture output
      console.log = (...args) => { stdout += args.join(' ') + '\n'; };
      console.error = (...args) => { stderr += args.join(' ') + '\n'; };

      // Save original Math.random
      const originalRandom = Math.random;
      Math.random = customRandom;

      try {
        // Create our sandbox/context
        const sandbox = {
          // Expose require so user code can require('pg'), etc.
          require,
          // Provide process if you want them to access certain environment vars or CPU usage
          process,
          // Overriding console so user logs go into our captured output
          console,
          // A fresh Math that only overrides random
          Math: Object.assign(Object.create(Math), { random: customRandom }),
          // If your user code needs setTimeout, Buffer, or others, include them
          setTimeout,
          Buffer,
          // If you want to allow top-level variables, e.g. 'Client' from pg
          // you can either let them do `const { Client } = require('pg')`
          // or you can inject it directly. Example:
          // Client: require('pg').Client
        };

        // Make the sandbox a real VM context
        vm.createContext(sandbox);

        // Wrap user code in an async IIFE so we get a Promise back
        // and so that top-level awaits or async calls are handled.
        const asyncWrapper = `
          (async () => {
            try {
              ${code}
            } catch (err) {
              console.error('Unhandled error:', err);
            }
          })()
        `;

        // Compile the script (with a big but not infinite timeout for synchronous parts)
        const script = new vm.Script(asyncWrapper, { timeout: 5000 });

        // Run it in the context
        const resultPromise = script.runInContext(sandbox);

        // Now 'resultPromise' is a Promise that will resolve once user’s async code finishes.
        // We can await that or do .then()
        await Promise.race([
          resultPromise,
          new Promise((_, reject) =>
            setTimeout(() => reject(new Error('User code timed out')), 60000)
          )
        ]);

      } catch (err) {
        stderr += (err.stack || err.toString()) + '\n';
        exitCode = 1;
      } finally {
        // Restore console
        console.log = originalLog;
        console.error = originalError;
        // Restore Math.random
        Math.random = originalRandom;
      }

      // Send response
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

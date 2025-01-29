// node-runner.js
const http = require('http');
const vm = require('vm');
const crypto = require('crypto');

let globalEntropy = crypto.randomBytes(32);

function resetRandomness(entropy) {
  globalEntropy = Buffer.from(entropy, 'hex');
  crypto.randomFill(globalEntropy, (err, buf) => {
    if (err) throw err;
    crypto.setRandomValues(() => buf);
  });

  // Optionally reset Math.random()
  const seed = crypto.createHash('sha256').update(globalEntropy).digest('hex');
  Math.random = require('seedrandom')(seed);
}

const server = http.createServer(async (req, res) => {
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

      if (!entropy || typeof entropy !== 'string' || entropy.length !== 64) {
        res.writeHead(400, { 'Content-Type': 'application/json' });
        return res.end(JSON.stringify({ error: 'Invalid entropy provided' }));
      }

      resetRandomness(entropy);

      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ message: 'Entropy reset successfully' }));
    });
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
        const sandboxConsole = {
          log: (...args) => { stdout += args.join(' ') + '\n'; },
          error: (...args) => { stderr += args.join(' ') + '\n'; }
        };

	//TODO: Add other packages
        const sandbox = { 
          console: sandboxConsole, 
          result: null,
          crypto: crypto,
          Math: Math
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

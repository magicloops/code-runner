// node-runner.js
const http = require('http');
const vm = require('vm');

const server = http.createServer(async (req, res) => {
  if (req.method === 'POST' && req.url === '/run') {
    let body = [];
    req.on('data', chunk => body.push(chunk));
    req.on('end', () => {
      // Parse JSON body of the form { "code": "...js code..." }
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
        // Optionally capture console output by overriding console.log/error
        const sandboxConsole = {
          log: (...args) => { stdout += args.join(' ') + '\n'; },
          error: (...args) => { stderr += args.join(' ') + '\n'; }
        };

        // Create a sandboxed context
        const sandbox = { console: sandboxConsole, result: null };

        // Run the code in a new context
        vm.runInNewContext(code, sandbox, { timeout: 360000 }); 
        // ^ The timeout helps avoid infinite loops, but be aware it’s not bulletproof
        //   for all CPU-bound or asynchronous tasks.

        // If you want the final value of the script, store it in sandbox.result
        // e.g. "sandbox.result = (function(){ ... your code... })()" 
        // but for now, we rely on console.log for "stdout"
      } catch (err) {
        // On error, keep the stack trace in stderr
        stderr += (err.stack || err.toString()) + '\n';
        // You could set a non-zero exitCode if desired
        exitCode = 1;
      }

      // Return JSON with stdout, stderr, exit_code
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

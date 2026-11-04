import { spawn } from 'child_process';

const PORT = parseInt(process.env.PORT || '8080', 10);

function executeRedditAgent(args: string[]): Promise<{ stdout: string; stderr: string; code: number }> {
  return new Promise((resolve) => {
    const child = spawn('reddit-agent', args, {
      env: { ...process.env },
      stdio: ['ignore', 'pipe', 'pipe']
    });

    let stdout = '';
    let stderr = '';

    child.stdout.on('data', (d) => { stdout += d.toString(); });
    child.stderr.on('data', (d) => { stderr += d.toString(); });

    child.on('close', (code) => {
      resolve({ stdout, stderr, code: code ?? 0 });
    });
  });
}

const server = Bun.serve({
  port: PORT,
  async fetch(req) {
    const url = new URL(req.url);

    // Health check endpoint for Cloud Run
    if (url.pathname === '/health' || url.pathname === '/') {
      return new Response(JSON.stringify({ status: 'ok', service: 'reddit-agent-cloud-run' }), {
        headers: { 'Content-Type': 'application/json' }
      });
    }

    // Cron pulse endpoint invoked periodically by Google Cloud Scheduler
    if (url.pathname === '/cron/pulse' && req.method === 'POST') {
      const body = await req.json().catch(() => ({}));
      const subreddits = (body as any).subreddits || 'vibecoding,startups,technology,programming';

      console.log(`[Cloud Run] Pulse triggered for subreddits: ${subreddits}`);

      const result = await executeRedditAgent([
        'opportunity',
        'discover',
        '--subreddit',
        subreddits.split(',')[0].trim(),
        '--limit',
        '3'
      ]);

      return new Response(JSON.stringify({
        status: result.code === 0 ? 'success' : 'error',
        exit_code: result.code,
        output: result.stdout,
        error: result.stderr,
        timestamp: new Date().toISOString()
      }), {
        headers: { 'Content-Type': 'application/json' },
        status: result.code === 0 ? 200 : 500
      });
    }

    return new Response(JSON.stringify({ error: 'Not Found' }), {
      status: 404,
      headers: { 'Content-Type': 'application/json' }
    });
  }
});

console.log(`[reddit-agent] Cloud Run agent listening on port ${PORT}`);

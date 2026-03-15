import { createServer } from 'node:http';
import { spawnSync, spawn } from 'node:child_process';
import { readdirSync, existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const SITES_DIR = '/app/astro-sites';
const GIT_REPOS_DIR = '/app/git-repos';
const PORT = parseInt(process.env.MANAGEMENT_PORT ?? '4320', 10);
const PREVIEW_PORT = parseInt(process.env.PREVIEW_PORT ?? '4321', 10);

/** @type {{ slug: string, process: import('child_process').ChildProcess } | null} */
let activePreview = null;

/** Serialises preview operations so concurrent requests don't race. */
let previewLock = Promise.resolve();

/** Poll localhost:{port} every 200ms until it responds or timeoutMs elapses. */
async function pollPort(port, timeoutMs = 10_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const res = await fetch(`http://localhost:${port}`);
      await res.body?.cancel(); // release the connection
      return; // server responded — we're done
    } catch {
      await new Promise((resolve) => setTimeout(resolve, 200));
    }
  }
  throw new Error(`Port ${port} not ready after ${timeoutMs}ms`);
}

/** Kill the active dev server and wait for the process to close (prevents port conflicts). */
async function stopActivePreview() {
  if (!activePreview) return;
  const proc = activePreview.process;
  activePreview = null; // clear state before awaiting so concurrent calls see it gone
  if (proc.exitCode !== null || proc.killed) return; // already dead
  await new Promise((resolve) => {
    proc.once('close', resolve);
    proc.kill('SIGTERM');
  });
}

/**
 * Start the Astro dev server for the given slug.
 * Returns { status, body } — body is SiteData (with previewUrl) on success,
 * or an RFC 9457 problem object on error.
 * Acquires the module-level lock so concurrent calls are serialised.
 */
async function startPreview(slug) {
  // Acquire the lock — wait for any in-flight preview operation to finish first.
  const result = await (previewLock = previewLock.then(() => _startPreview(slug)));
  return result;
}

async function _startPreview(slug) {
  const siteDir = join(SITES_DIR, slug);
  if (!existsSync(siteDir)) {
    return {
      status: 404,
      body: { type: 'about:blank', title: 'Not Found', status: 404, detail: `Site '${slug}' does not exist` },
    };
  }

  const metaPath = join(siteDir, '.cms-meta.json');
  const name = existsSync(metaPath) ? (JSON.parse(readFileSync(metaPath, 'utf8')).name ?? slug) : slug;
  const gitUrl = join(GIT_REPOS_DIR, `${slug}.git`);

  await stopActivePreview();

  const proc = spawn('pnpm', ['dev', '--host', '0.0.0.0'], { cwd: siteDir, detached: false });

  try {
    await pollPort(PREVIEW_PORT);
  } catch (err) {
    await new Promise((resolve) => {
      proc.once('close', resolve);
      proc.kill('SIGTERM');
    });
    return {
      status: 500,
      body: { type: 'about:blank', title: 'Dev Server Failed', status: 500, detail: err.message },
    };
  }

  activePreview = { slug, process: proc };
  const previewUrl = `http://localhost:${PREVIEW_PORT}`;
  return { status: 200, body: { slug, name, gitUrl, previewUrl } };
}

function run(cmd, args, cwd) {
  const result = spawnSync(cmd, args, { cwd, stdio: 'pipe', encoding: 'utf8' });
  if (result.status !== 0) {
    throw new Error(`Command '${cmd} ${args.join(' ')}' failed:\n${result.stderr}`);
  }
  return result.stdout;
}

function readJson(body) {
  return new Promise((resolve, reject) => {
    let data = '';
    body.on('data', (chunk) => {
      data += chunk;
    });
    body.on('end', () => {
      try {
        resolve(JSON.parse(data));
      } catch {
        reject(new Error('Invalid JSON'));
      }
    });
  });
}

function respond(res, status, body) {
  res.writeHead(status, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify(body));
}

function listSites() {
  if (!existsSync(SITES_DIR)) return [];
  return readdirSync(SITES_DIR, { withFileTypes: true })
    .filter((d) => d.isDirectory())
    .map((d) => {
      const slug = d.name;
      const metaPath = join(SITES_DIR, slug, '.cms-meta.json');
      const name = existsSync(metaPath) ? (JSON.parse(readFileSync(metaPath, 'utf8')).name ?? slug) : slug;
      const gitUrl = join(GIT_REPOS_DIR, `${slug}.git`);
      const previewUrl = activePreview?.slug === slug ? `http://localhost:${PREVIEW_PORT}` : null;
      return { slug, name, gitUrl, previewUrl };
    });
}

function createSite({ name, slug }) {
  const siteDir = join(SITES_DIR, slug);
  const bareRepo = join(GIT_REPOS_DIR, `${slug}.git`);

  if (existsSync(siteDir)) {
    throw new Error(`Site '${slug}' already exists`);
  }

  // Scaffold the Astro project
  run(
    'pnpm',
    ['create', 'astro@latest', siteDir, '--template', 'minimal', '--no-git', '--skip-houston', '--no-install'],
    undefined,
  );
  run('pnpm', ['install'], siteDir);

  // Persist human-readable name (package.json.name is the slug, not the display name)
  writeFileSync(join(siteDir, '.cms-meta.json'), JSON.stringify({ name, slug }));

  // Initialise git and commit
  run('git', ['config', '--global', 'init.defaultBranch', 'main'], undefined);
  run('git', ['init'], siteDir);
  run('git', ['config', 'user.email', 'cms@blog-engine.local'], siteDir);
  run('git', ['config', 'user.name', 'Blog Engine CMS'], siteDir);
  run('git', ['add', '.'], siteDir);
  run('git', ['commit', '-m', `Initial Astro site: ${name}`], siteDir);

  // Create local bare repo and push
  run('git', ['init', '--bare', bareRepo], undefined);
  run('git', ['remote', 'add', 'origin', bareRepo], siteDir);
  run('git', ['push', '-u', 'origin', 'main'], siteDir);

  return { slug, name, gitUrl: bareRepo };
}

const server = createServer(async (req, res) => {
  const url = new URL(req.url, `http://localhost:${PORT}`);

  // GET /sites
  if (req.method === 'GET' && url.pathname === '/sites') {
    respond(res, 200, listSites());
    return;
  }

  // POST /sites
  if (req.method === 'POST' && url.pathname === '/sites') {
    try {
      const { name, slug } = await readJson(req);
      if (!name || !slug) {
        respond(res, 400, { error: 'name and slug are required' });
        return;
      }
      const site = createSite({ name, slug });
      respond(res, 201, site);
    } catch (err) {
      respond(res, 500, { error: err.message });
    }
    return;
  }

  // POST /sites/:slug/preview — start dev server for the given site
  const previewMatch = url.pathname.match(/^\/sites\/([^/]+)\/preview$/);
  if (req.method === 'POST' && previewMatch) {
    const slug = previewMatch[1];
    const result = await startPreview(slug);
    respond(res, result.status, result.body);
    return;
  }

  // DELETE /preview — stop the active dev server (idempotent)
  if (req.method === 'DELETE' && url.pathname === '/preview') {
    await stopActivePreview();
    res.writeHead(204);
    res.end();
    return;
  }

  respond(res, 404, { error: 'Not found' });
});

server.listen(PORT, () => {
  console.log(`Management API listening on port ${PORT}`);
});

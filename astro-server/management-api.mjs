import { createServer } from 'node:http';
import { spawnSync } from 'node:child_process';
import { readdirSync, existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const SITES_DIR = '/app/astro-sites';
const GIT_REPOS_DIR = '/app/git-repos';
const PORT = parseInt(process.env.MANAGEMENT_PORT ?? '4320', 10);

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
      return { slug, name, gitUrl };
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

  respond(res, 404, { error: 'Not found' });
});

server.listen(PORT, () => {
  console.log(`Management API listening on port ${PORT}`);
});

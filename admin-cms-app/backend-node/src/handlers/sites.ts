import type { FastifyRequest, FastifyReply } from 'fastify';
import { randomUUID } from 'node:crypto';
import packageJson from '../../package.json' with { type: 'json' };
import type { components } from '../generated-api-types.js';

type SiteData = components['schemas']['SiteData'];
type CreateSiteRequest = components['schemas']['CreateSiteRequest'];
type MetaServerName = components['schemas']['MetaServerName'];

const ASTRO_MANAGEMENT_URL = process.env.ASTRO_MANAGEMENT_URL ?? 'http://localhost:4320';

function makeMeta() {
  return {
    requestId: randomUUID(),
    serverName: packageJson.name as MetaServerName,
    timestamp: new Date().toISOString(),
    version: packageJson.version,
  };
}

export async function list_sites(_request: FastifyRequest, reply: FastifyReply) {
  const resp = await fetch(`${ASTRO_MANAGEMENT_URL}/sites`);
  const sites: SiteData[] = await resp.json();
  return reply.code(200).send({ data: sites, meta: makeMeta() });
}

export async function create_site(request: FastifyRequest<{ Body: CreateSiteRequest }>, reply: FastifyReply) {
  const resp = await fetch(`${ASTRO_MANAGEMENT_URL}/sites`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(request.body),
  });

  if (!resp.ok) {
    const err: unknown = await resp.json();
    return reply.code(500).send(err);
  }

  const site: SiteData = await resp.json();
  return reply.code(201).send({ data: site, meta: makeMeta() });
}

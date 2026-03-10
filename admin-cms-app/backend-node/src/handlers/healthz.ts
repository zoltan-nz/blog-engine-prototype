import type { FastifyRequest, FastifyReply } from 'fastify';
import { randomUUID } from 'node:crypto';
import packageJson from '../../package.json' with { type: 'json' };
import type { components } from '../generated-api-types.js';

type HealthzResponse = components['schemas']['Envelop'];
type MetaServerName = components['schemas']['MetaServerName'];

export async function healthz(_request: FastifyRequest, _reply: FastifyReply): Promise<HealthzResponse> {
  return {
    data: {
      status: 'healthy' as const,
      version: packageJson.version,
    },
    meta: {
      requestId: randomUUID(),
      serverName: packageJson.name as MetaServerName,
      timestamp: new Date().toISOString(),
      version: packageJson.version,
    },
  };
}

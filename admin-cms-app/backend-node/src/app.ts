import Fastify, { type FastifyBaseLogger, type FastifyHttpOptions } from 'fastify';
import * as https from 'node:https';
import cors from '@fastify/cors';
import type { HealthResponse } from './types.js';
import { randomUUID } from 'node:crypto';
import packageJson from '../package.json' with { type: 'json' };

type FastifyOptions = FastifyHttpOptions<https.Server, FastifyBaseLogger>;

const buildFastifyApp = async (opts?: FastifyOptions) => {
  const app = Fastify(opts);
  await app.register(cors);

  app.get('/healthz', async (): Promise<HealthResponse> => {
    return {
      data: {
        status: 'healthy',
      },
      meta: {
        timestamp: new Date().toISOString(),
        requestId: randomUUID(),
        serverName: packageJson.name,
        version: packageJson.version,
      },
    };
  });

  return app;
};

export { buildFastifyApp };
export type { FastifyOptions };

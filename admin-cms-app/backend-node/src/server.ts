import Fastify from 'fastify';
import cors from '@fastify/cors';
import type { HealthResponse } from './types.js';
import { randomUUID } from 'node:crypto';

const fastify = Fastify({ logger: true });
await fastify.register(cors);

fastify.get('/healthz', async (): Promise<HealthResponse> => {
  return {
    data: {
      status: 'healthy',
      version: '0.1.0',
    },
    meta: {
      timestamp: new Date().toISOString(),
      requestId: randomUUID(),
    },
  };
});

const start = async () => {
  try {
    await fastify.listen({ port: 8080, host: '0.0.0.0' });
  } catch (err) {
    fastify.log.error(err);
    process.exit(1);
  }
};

await start();

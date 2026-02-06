import { beforeEach, expect, test } from 'vitest';
import { buildFastifyApp } from './app.js';
import type { FastifyInstance } from 'fastify';
import packageJson from '../package.json' with { type: 'json' };

let app: FastifyInstance;

beforeEach(async () => {
  app = await buildFastifyApp();
});

test('GET /healthz', async () => {
  const response = await app.inject({
    method: 'GET',
    url: '/healthz',
  });
  expect(response.statusCode).toBe(200);
  expect(response.json()).toEqual({
    data: {
      status: 'healthy',
    },
    meta: {
      timestamp: expect.any(String),
      requestId: expect.any(String),
      serverName: packageJson.name,
      version: packageJson.version,
    },
  });
});

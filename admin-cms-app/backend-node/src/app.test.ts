import { afterEach, beforeEach, expect, test, describe } from 'vitest';
import { buildFastifyApp } from './app.js';
import type { FastifyInstance } from 'fastify';
import packageJson from '../package.json' with { type: 'json' };

let app: FastifyInstance;

beforeEach(async () => {
  app = await buildFastifyApp();
});

afterEach(async () => {
  await app.close();
});

describe('GET /healthz', () => {
  test('returns 200 with a valid envelope', async () => {
    const response = await app.inject({ method: 'GET', url: '/healthz' });

    expect(response.statusCode).toBe(200);
    expect(response.json()).toEqual({
      data: {
        status: 'healthy',
        version: packageJson.version,
      },
      meta: {
        timestamp: expect.any(String),
        requestId: expect.any(String),
        serverName: packageJson.name,
        version: packageJson.version,
      },
    });
  });

  test('response shape is validated by Zod — rejects unknown routes', async () => {
    const response = await app.inject({ method: 'GET', url: '/unknown' });
    expect(response.statusCode).toBe(404);
  });

  test('meta.timestamp is a valid ISO 8601 string', async () => {
    const response = await app.inject({ method: 'GET', url: '/healthz' });
    const { meta } = response.json();
    expect(new Date(meta.timestamp).toISOString()).toBe(meta.timestamp);
  });

  test('meta.requestId is a valid UUID', async () => {
    const response = await app.inject({ method: 'GET', url: '/healthz' });
    const { meta } = response.json();
    expect(meta.requestId).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
  });

  test('each request gets a unique requestId', async () => {
    const [r1, r2] = await Promise.all([
      app.inject({ method: 'GET', url: '/healthz' }),
      app.inject({ method: 'GET', url: '/healthz' }),
    ]);
    expect(r1.json().meta.requestId).not.toBe(r2.json().meta.requestId);
  });
});

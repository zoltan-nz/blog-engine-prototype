import Fastify, { type FastifyBaseLogger, type FastifyHttpOptions } from 'fastify';
import * as https from 'node:https';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import cors from '@fastify/cors';
import openapiGlue from 'fastify-openapi-glue';
import { healthz } from './handlers/healthz.js';
import { list_sites, create_site } from './handlers/sites.js';

type FastifyOptions = FastifyHttpOptions<https.Server, FastifyBaseLogger>;

const __dirname = dirname(fileURLToPath(import.meta.url));
const specPath = join(__dirname, '../../../open-api-contracts/api.yaml');

const serviceHandlers = { healthz, list_sites, create_site };

const buildFastifyApp = async (opts?: FastifyOptions) => {
  const app = Fastify({
    ...opts,
    ajv: {
      customOptions: { strict: false },
    },
  });

  await app.register(cors);

  await app.register(openapiGlue, {
    specification: specPath,
    serviceHandlers,
  });

  return app;
};

export { buildFastifyApp };
export type { FastifyOptions };

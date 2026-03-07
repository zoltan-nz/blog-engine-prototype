import Fastify, { type FastifyBaseLogger, type FastifyHttpOptions } from 'fastify';
import * as https from 'node:https';
import cors from '@fastify/cors';
import { serializerCompiler, validatorCompiler, type ZodTypeProvider } from 'fastify-type-provider-zod';
import { randomUUID } from 'node:crypto';
import packageJson from '../package.json' with { type: 'json' };
import { schemas } from './generated-schemas.js';

type FastifyOptions = FastifyHttpOptions<https.Server, FastifyBaseLogger>;

const buildFastifyApp = async (opts?: FastifyOptions) => {
  const app = Fastify(opts);

  app.setValidatorCompiler(validatorCompiler);
  app.setSerializerCompiler(serializerCompiler);

  await app.register(cors);

  app.withTypeProvider<ZodTypeProvider>().get(
    '/healthz',
    {
      schema: {
        response: {
          200: schemas.Envelop,
        },
      },
    },
    async () => {
      return {
        data: {
          status: 'healthy' as const,
          version: packageJson.version,
        },
        meta: {
          timestamp: new Date().toISOString(),
          requestId: randomUUID(),
          serverName: packageJson.name as 'backend-node',
          version: packageJson.version,
        },
      };
    },
  );

  return app;
};

export { buildFastifyApp };
export type { FastifyOptions };

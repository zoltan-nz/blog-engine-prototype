import { buildFastifyApp, type FastifyOptions } from './app.js';

const DEFAULT_HOST = '0.0.0.0';
const DEFAULT_PORT = '8081';

const start = async () => {
  const opts: FastifyOptions = {
    logger: {
      level: 'info',
      transport: {
        target: 'pino-pretty',
        options: {
          translateTime: 'HH:MM:ss Z',
          ignore: 'pid,hostname',
        },
      },
    },
  };

  const app = await buildFastifyApp(opts);

  const port: number = parseInt(process.env.VITE_PORT || process.env.PORT || DEFAULT_PORT, 10);

  try {
    await app.listen({ port, host: DEFAULT_HOST });
  } catch (err) {
    app.log.error(err);
    process.exit(1);
  }
};

await start();

export { start };

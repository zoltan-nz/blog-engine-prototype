// ⚠️ Auto-generated from open-api-contracts/api.yaml — do not edit manually
// Run: pnpm gen:schemas
import { z } from 'zod';

export const HealthStatus = z.literal('healthy');
export const HealthData = z.object({ status: HealthStatus, version: z.string() }).passthrough();
export const MetaServerName = z.enum(['backend-node', 'backend-rust']);
export const Meta = z
  .object({ requestId: z.string(), serverName: MetaServerName, timestamp: z.string(), version: z.string() })
  .passthrough();
export const Envelop = z.object({ data: HealthData, meta: Meta }).passthrough();

export const schemas = {
  HealthStatus,
  HealthData,
  MetaServerName,
  Meta,
  Envelop,
};

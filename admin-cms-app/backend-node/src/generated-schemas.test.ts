import { describe, expect, test } from 'vitest';
import { schemas } from './generated-schemas.js';

describe('HealthStatus', () => {
  test('accepts "healthy"', () => {
    expect(schemas.HealthStatus.parse('healthy')).toBe('healthy');
  });

  test('rejects other strings', () => {
    expect(() => schemas.HealthStatus.parse('degraded')).toThrow();
    expect(() => schemas.HealthStatus.parse('')).toThrow();
  });
});

describe('MetaServerName', () => {
  test('accepts valid server names', () => {
    expect(schemas.MetaServerName.parse('backend-node')).toBe('backend-node');
    expect(schemas.MetaServerName.parse('backend-rust')).toBe('backend-rust');
  });

  test('rejects unknown server names', () => {
    expect(() => schemas.MetaServerName.parse('backend-python')).toThrow();
    expect(() => schemas.MetaServerName.parse('')).toThrow();
  });
});

describe('HealthData', () => {
  test('accepts valid health data', () => {
    const result = schemas.HealthData.parse({ status: 'healthy', version: '1.0.0' });
    expect(result).toEqual({ status: 'healthy', version: '1.0.0' });
  });

  test('rejects missing fields', () => {
    expect(() => schemas.HealthData.parse({ status: 'healthy' })).toThrow();
    expect(() => schemas.HealthData.parse({ version: '1.0.0' })).toThrow();
  });

  test('rejects invalid status', () => {
    expect(() => schemas.HealthData.parse({ status: 'unknown', version: '1.0.0' })).toThrow();
  });
});

describe('Meta', () => {
  const validMeta = {
    requestId: '123e4567-e89b-12d3-a456-426614174000',
    serverName: 'backend-node',
    timestamp: new Date().toISOString(),
    version: '1.0.0',
  };

  test('accepts valid meta', () => {
    expect(schemas.Meta.parse(validMeta)).toEqual(validMeta);
  });

  test('rejects missing required fields', () => {
    const { requestId: _, ...withoutRequestId } = validMeta;
    expect(() => schemas.Meta.parse(withoutRequestId)).toThrow();
  });

  test('rejects invalid serverName', () => {
    expect(() => schemas.Meta.parse({ ...validMeta, serverName: 'unknown' })).toThrow();
  });
});

describe('Envelop', () => {
  const validEnvelop = {
    data: { status: 'healthy', version: '1.0.0' },
    meta: {
      requestId: '123e4567-e89b-12d3-a456-426614174000',
      serverName: 'backend-node',
      timestamp: new Date().toISOString(),
      version: '1.0.0',
    },
  };

  test('accepts a valid envelop', () => {
    expect(schemas.Envelop.parse(validEnvelop)).toEqual(validEnvelop);
  });

  test('rejects missing data', () => {
    const { data: _, ...withoutData } = validEnvelop;
    expect(() => schemas.Envelop.parse(withoutData)).toThrow();
  });

  test('rejects missing meta', () => {
    const { meta: _, ...withoutMeta } = validEnvelop;
    expect(() => schemas.Envelop.parse(withoutMeta)).toThrow();
  });
});

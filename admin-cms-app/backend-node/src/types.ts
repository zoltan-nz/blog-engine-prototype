export interface Meta {
  timestamp: string;
  requestId: string;
  serverName: string;
  version: string;
}

export interface Envelope<T> {
  data: T;
  meta: Meta;
}

// Health endpoint
export type HealthStatus = 'healthy' | 'degraded' | 'unhealthy';

export interface HealthData {
  status: HealthStatus;
}

export type HealthResponse = Envelope<HealthData>;

// RFC 7807 Problem Details
export interface ProblemDetails {
  type: string;
  title: string;
  status: number;
  detail?: string;
  instance?: string;
  errors?: ProblemDetailsError[];
}

export interface ProblemDetailsError {
  field: string;
  message: string;
}

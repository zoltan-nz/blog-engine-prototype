export interface Meta {
  timestamp: string;
  requestId: string;
  serverName: string;
  version: string;
}

export interface Envelop<T> {
  data: T;
  meta: Meta;
}

// Health endpoint
export type HealthStatus = 'healthy';

export interface HealthData {
  status: HealthStatus;
}

export type HealthResponse = Envelop<HealthData>;

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

import { env } from "$env/dynamic/public";

export const backendURL = env.PUBLIC_API_BACKEND_URL || "http://localhost:8080";

export const fetchWithServerUrl = async <T>(
  url: string,
  options: RequestInit,
): Promise<T> => {
  const requestUrl = `${backendURL}${url}`;

  console.log(`fetchWithServerUrl was called with url: ${backendURL}${url}`);

  const request = new Request(requestUrl, options);
  const response = await fetch(request);
  const data = await response.json();

  return { status: response.status, data } as T;
};

export const backendURL =
  import.meta.env.VITE_API_BACKEND_URL || "http://localhost:3001";

export const fetchWithBaseUrl = async <T>(
  url: string,
  options: RequestInit,
): Promise<T> => {
  const requestUrl = `${backendURL}${url}`;
  const response = await fetch(requestUrl, options);
  const data = await response.json();

  return { status: response.status, data } as T;
};

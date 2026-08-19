export function safeLocalDemoApiPath(segments: string[]): string | undefined;

export function fetchLocalDemoApi(options: {
  apiBaseUrl: string;
  segments: string[];
  search: string;
  method: "GET" | "POST";
  headers: Headers;
  fetchImpl?: typeof fetch;
}): Promise<Response>;

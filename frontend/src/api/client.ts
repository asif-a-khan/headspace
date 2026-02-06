/** API client with CSRF token handling. */

function getCsrfToken(): string {
  const meta = document.querySelector('meta[name="csrf-token"]');
  return meta?.getAttribute("content") ?? "";
}

interface RequestOptions {
  method?: string;
  body?: unknown;
  headers?: Record<string, string>;
}

export async function api<T = unknown>(
  url: string,
  options: RequestOptions = {}
): Promise<T> {
  const { method = "GET", body, headers = {} } = options;

  const fetchHeaders: Record<string, string> = {
    Accept: "application/json",
    ...headers,
  };

  if (method !== "GET" && method !== "HEAD") {
    fetchHeaders["X-CSRF-Token"] = getCsrfToken();
  }

  const fetchOptions: RequestInit = {
    method,
    headers: fetchHeaders,
    credentials: "same-origin",
  };

  if (body !== undefined) {
    fetchHeaders["Content-Type"] = "application/json";
    fetchOptions.body = JSON.stringify(body);
  }

  const response = await fetch(url, fetchOptions);

  if (!response.ok) {
    const text = await response.text();
    let message: string;
    try {
      const json = JSON.parse(text);
      message = json.error || json.message || text;
    } catch {
      message = text;
    }
    throw new ApiError(response.status, message);
  }

  const text = await response.text();
  if (!text) return undefined as T;
  return JSON.parse(text) as T;
}

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string
  ) {
    super(message);
    this.name = "ApiError";
  }
}

// Convenience methods
export const get = <T = unknown>(url: string) => api<T>(url);
export const post = <T = unknown>(url: string, body?: unknown) =>
  api<T>(url, { method: "POST", body });
export const put = <T = unknown>(url: string, body?: unknown) =>
  api<T>(url, { method: "PUT", body });
export const del = <T = unknown>(url: string) =>
  api<T>(url, { method: "DELETE" });

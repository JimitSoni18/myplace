declare const __APP_TARGET__: string | undefined;

export function isOwnerApp(): boolean {
  if (typeof __APP_TARGET__ !== "undefined") {
    return __APP_TARGET__ === "owner";
  }
  if (typeof window !== "undefined") {
    return window.location.port === "3002" || window.location.pathname.includes("owner");
  }
  return false;
}

export const getTokenKey = () => (isOwnerApp() ? "owner_auth_token" : "admin_auth_token");
export const getUserKey = () => (isOwnerApp() ? "owner_user_info" : "admin_user_info");
export const TOKEN_KEY = "admin_auth_token";
export const USER_KEY = "admin_user_info";

export const BASE_URL = import.meta.env.VITE_API_URL || "";

export class ApiError extends Error {
  status: number;
  code?: string;
  data?: unknown;

  constructor(status: number, message: string, code?: string, data?: unknown) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.code = code;
    this.data = data;
  }
}

export function getToken(): string | null {
  return localStorage.getItem(getTokenKey());
}

export function setToken(token: string): void {
  localStorage.setItem(getTokenKey(), token);
}

export function clearToken(): void {
  localStorage.removeItem(getTokenKey());
  localStorage.removeItem(getUserKey());
  localStorage.removeItem("myplace_auth_token");
  localStorage.removeItem("myplace_user_info");
}

export function getCurrentUser() {
  const raw = localStorage.getItem(getUserKey());
  if (!raw) return null;
  try {
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

export function setCurrentUser(user: unknown): void {
  localStorage.setItem(getUserKey(), JSON.stringify(user));
}

async function request<T>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const url = `${BASE_URL}${path}`;
  const headers = new Headers(options.headers || {});

  const token = getToken();
  if (token && !headers.has("Authorization")) {
    headers.set("Authorization", `Bearer ${token}`);
  }

  if (!headers.has("Accept")) {
    headers.set("Accept", "application/json");
  }

  const response = await fetch(url, {
    ...options,
    headers,
    credentials: "include",
  });

  if (response.status === 401) {
    clearToken();
    // Redirect to login if unauthenticated on protected routes
    if (typeof window !== "undefined" && !window.location.hash.includes("/login")) {
      window.location.hash = "/login";
    }
    throw new ApiError(401, "Session expired or unauthorized", "UNAUTHORIZED");
  }


  if (response.status === 204) {
    return null as T;
  }

  const contentType = response.headers.get("content-type");
  let body: unknown = null;
  if (contentType && contentType.includes("application/json")) {
    body = await response.json();
  } else {
    body = await response.text();
  }

  if (!response.ok) {
    const errorData = typeof body === "object" && body !== null ? (body as Record<string, unknown>) : {};
    const message = (errorData.message as string) || response.statusText || "Request failed";
    const code = (errorData.code as string) || "UNKNOWN_ERROR";
    throw new ApiError(response.status, message, code, body);
  }

  return body as T;
}

export const api = {
  get<T>(path: string, headers?: HeadersInit): Promise<T> {
    return request<T>(path, { method: "GET", headers });
  },

  post<T>(path: string, body?: unknown, headers?: HeadersInit): Promise<T> {
    const isFormData = body instanceof FormData;
    const finalHeaders = new Headers(headers);
    if (!isFormData && !finalHeaders.has("Content-Type")) {
      finalHeaders.set("Content-Type", "application/json");
    }

    return request<T>(path, {
      method: "POST",
      headers: finalHeaders,
      body: isFormData ? body : JSON.stringify(body),
    });
  },

  put<T>(path: string, body?: unknown, headers?: HeadersInit): Promise<T> {
    const isFormData = body instanceof FormData;
    const finalHeaders = new Headers(headers);
    if (!isFormData && !finalHeaders.has("Content-Type")) {
      finalHeaders.set("Content-Type", "application/json");
    }

    return request<T>(path, {
      method: "PUT",
      headers: finalHeaders,
      body: isFormData ? body : JSON.stringify(body),
    });
  },

  delete<T>(path: string, headers?: HeadersInit): Promise<T> {
    return request<T>(path, { method: "DELETE", headers });
  },

  upload<T>(path: string, formData: FormData): Promise<T> {
    return request<T>(path, {
      method: "POST",
      body: formData,
    });
  },
};

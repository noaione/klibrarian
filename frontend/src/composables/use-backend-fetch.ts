import useAuth from "./use-auth";

function makeUrl(url: string): string {
  const baseHost = import.meta.env.VITE_BASE_HOST;

  if (url.startsWith("/")) {
    url = url.slice(1);
  }

  if (!url.startsWith("api/")) {
    url = `api/${url}`;
  }

  if (baseHost) {
    if (baseHost.endsWith("/")) {
      return `${baseHost}${url}`;
    }

    return `${baseHost}/${url}`;
  }

  return `/${url}`;
}

export default function useBackendFetch<T>(url: string, fetchOptions?: RequestInit): Promise<T> {
  const auth = useAuth();

  const headers = new Headers(fetchOptions?.headers);

  if (auth.token) {
    headers.set("Authorization", `Bearer ${auth.token}`);
  }

  const mergedFetchOptions: RequestInit = {
    ...fetchOptions,
    headers,
  };

  return new Promise<T>((resolve, reject) => {
    fetch(makeUrl(url), mergedFetchOptions)
      .then((resp) => {
        if (resp.ok) {
          return resp.json();
        }

        throw new Error(resp.statusText);
      })
      .then((json) => {
        if (json.ok) {
          resolve(json.data);
        } else {
          reject(json.error);
        }
      })
      .catch((error) => {
        reject(error);
      });
  });
}

import { defineStore } from "pinia";

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

const useAuth = defineStore(
  "librarian.auth",
  () => {
    const token = ref<string>();

    const isLoggedIn = computed(() => !!token.value);

    async function test() {
      try {
        const resp = await fetch(makeUrl("/api/invite/info"), {
          headers: {
            Authorization: `Bearer ${token.value}`,
          },
        });
        const data = await resp.json();

        if (!data.ok) {
          token.value = undefined;

          throw new Error(data.error);
        }
      } catch (error) {
        console.error(error);

        token.value = undefined;

        throw error;
      }
    }

    async function login(loginToken: string) {
      // test with api
      try {
        const resp = await fetch(makeUrl("/api/auth/login"), {
          method: "POST",
          body: JSON.stringify({ token: loginToken }),
          headers: {
            "Content-Type": "application/json",
          },
        });
        const data = await resp.json();

        if (data.ok) {
          token.value = loginToken;
        } else {
          throw new Error(data.error);
        }
      } catch (error) {
        console.error(error);

        throw error;
      }
    }

    function logout() {
      token.value = undefined;
    }

    return {
      token,
      isLoggedIn,
      login,
      logout,
      test,
    };
  },
  {
    persist: {
      key: "librarian.auth",
      storage: localStorage,
    },
  }
);

export default useAuth;

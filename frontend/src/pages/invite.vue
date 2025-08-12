<template>
  <main class="mx-auto my-auto flex h-screen w-full flex-col items-center justify-center">
    <i-mdi-key-chain class="mb-2 h-12 w-12" />
    <div class="font-variable text-xl variation-weight-bold">K-Librarian</div>
    <hr v-if="inviteData" class="server-width my-4 border-gray-600 opacity-70 dark:border-gray-400" />
    <div v-if="inviteData && !registeredHost" class="server-width flex flex-col justify-start">
      <img
        :src="inviteData.kind === 'komga' ? '/komga-logo.svg' : '/navidrome-logo.png'"
        class="mx-auto mb-4 h-16 w-16 object-contain"
      />
      <span class="font-variable mb-1 text-center variation-weight-bold">Invite to {{ upper(inviteData.kind) }}</span>
      <span class="font-variable mb-2 text-center variation-weight-medium">{{ inviteData.token }}</span>
      <div class="flex w-full flex-col items-start gap-2">
        <div class="flex w-full flex-col">
          <label class="font-variable mb-1 text-sm variation-weight-medium">Username</label>
          <input
            v-model="username"
            type="text"
            class="form-input w-full transition disabled:cursor-not-allowed disabled:border-opacity-50 disabled:bg-gray-100 dark:bg-gray-800 disabled:dark:bg-gray-900"
            name="username"
            :disabled="submitting"
            required
          />
        </div>
        <div ref="validUserNameRef" class="server-width flex flex-col justify-start gap-1">
          <div v-for="(error, idx) in validationUsername" :key="idx" class="text-red-400">{{ error }}</div>
        </div>
        <div class="flex w-full flex-col">
          <label class="font-variable mb-1 text-sm variation-weight-medium">Email</label>
          <input
            v-model="email"
            type="email"
            class="form-input w-full transition disabled:cursor-not-allowed disabled:border-opacity-50 disabled:bg-gray-100 dark:bg-gray-800 disabled:dark:bg-gray-900"
            name="email"
            :disabled="submitting"
            required
          />
        </div>
        <div ref="validUserRef" class="server-width flex flex-col justify-start gap-1">
          <div v-for="(error, idx) in validationEmail" :key="idx" class="text-red-400">{{ error }}</div>
        </div>
        <div class="flex w-full flex-col">
          <label class="font-variable mb-1 text-sm variation-weight-medium">Password</label>
          <input
            v-model="password"
            type="password"
            class="form-input w-full transition disabled:cursor-not-allowed disabled:border-opacity-50 disabled:bg-gray-100 dark:bg-gray-800 disabled:dark:bg-gray-900"
            name="password"
            :disabled="submitting"
            required
          />
        </div>
        <div ref="validPassRef" class="server-width flex flex-col justify-start gap-1">
          <div v-for="(error, idx) in validationPassword" :key="idx" class="text-red-400">{{ error }}</div>
        </div>
        <div class="mt-2 flex w-full flex-row items-center justify-center">
          <button
            class="font-variable flex w-full flex-row items-center justify-center border-2 border-cyan-500 bg-transparent px-2 py-1.5 text-sm text-cyan-500 transition variation-weight-[550] hover:bg-cyan-600 hover:text-white disabled:cursor-not-allowed disabled:bg-cyan-600 disabled:text-white disabled:opacity-80"
            :disabled="submitting || hasValidationError"
            @click="register"
          >
            Register
          </button>
        </div>
      </div>
    </div>
    <div v-else-if="inviteData && registeredHost" class="server-width flex flex-col justify-start">
      <div class="mt-2 flex flex-row items-center">
        <i-mdi-check-circle class="mx-auto h-8 w-8 text-green-500" />
      </div>
      <span class="font-variable mt-2 text-center variation-weight-medium">You are now registered!</span>
      <span class="font-variable text-center variation-weight-medium">{{ inviteData.token }}</span>

      <div class="mt-4 flex flex-col items-center">
        <span>Email: {{ email }}</span>
        <span>Password: {{ password }}</span>

        <a
          :href="registeredHost"
          target="_blank"
          class="font-variable mt-4 flex flex-row items-center justify-center border-2 border-cyan-500 bg-transparent px-2 py-1.5 text-sm text-cyan-500 transition variation-weight-[550] hover:bg-cyan-600 hover:text-white disabled:cursor-not-allowed disabled:bg-cyan-600 disabled:text-white disabled:opacity-80"
        >
          Login to Komga
        </a>
      </div>
    </div>
    <div v-else class="server-width flex flex-col justify-start">
      <div class="mt-4 flex flex-row items-center">
        <i-mdi-loading class="mx-auto h-8 w-8 animate-spin" />
      </div>
    </div>
    <hr class="server-width my-4 border-gray-600 opacity-70 dark:border-gray-400" />
    <div class="mt-2 flex flex-row gap-2">
      <router-link to="/" class="transition hover:opacity-70 dark:hover:opacity-80">
        <i-mdi-home class="h-8 w-8" />
      </router-link>
      <router-link to="/admin" class="transition hover:opacity-70 dark:hover:opacity-80">
        <i-mdi-login class="h-8 w-8" />
      </router-link>
      <dark-toggle />
    </div>
  </main>
  <footer-info />
</template>

<script setup lang="ts">
import useBackendFetch from "@/composables/use-backend-fetch";
import useToast from "@/composables/use-toast";
import type { Invite } from "@/types/invites";
import autoAnimate from "@formkit/auto-animate";

interface SubmitResponse {
  host: string;
}

const inviteData = ref<Invite>();
const toast = useToast();
const submitting = ref(false);

const head = injectHead();

const registeredHost = ref<string>();

const validUserNameRef = ref();
const validUserRef = ref();
const validPassRef = ref();
const validationUsername = ref(["Username cannot be empty"]);
const validationEmail = ref(["Email cannot be empty"]);
const validationPassword = ref(["Password cannot be empty"]);

const username = ref("");
const email = ref("");
const password = ref("");
const hasValidationError = computed(
  () => validationEmail.value.length > 0 || validationPassword.value.length > 0 || validationUsername.value.length > 0
);

async function register() {
  if (hasValidationError.value) {
    return;
  }

  if (inviteData.value === undefined) {
    toast.toast({
      message: "Invalid invite link",
      type: "error",
    });

    return;
  }

  submitting.value = true;

  try {
    const data = await useBackendFetch<SubmitResponse>(`/invite/${inviteData.value?.token}/apply`, {
      method: "POST",
      body: JSON.stringify({
        email: email.value,
        password: password.value,
        username: username.value,
      }),
      headers: {
        "Content-Type": "application/json",
      },
    });

    registeredHost.value = data.host;

    toast.toast({
      title: "Registered",
      message: "You can now login",
      type: "success",
    });
  } catch (error) {
    if (error instanceof Error) {
      toast.toast({
        title: "Unknown error occured",
        message: error.message,
        type: "error",
      });
    } else {
      toast.toast({
        title: "Failed to register",
        message: String(error),
        type: "error",
      });
    }
  } finally {
    submitting.value = false;
  }
}

function isValidEmail(newMail: string) {
  const re =
    // eslint-disable-next-line no-control-regex
    /(?:[\d!#$%&'*+/=?^_`a-z{|}~-]+(?:\.[\d!#$%&'*+/=?^_`a-z{|}~-]+)*|"(?:[\u0001-\u0008\u000B\u000C\u000E-\u001F!\u0023-\u005B\u005D-\u007F]|\\[\u0001-\u0009\u000B\u000C\u000E-\u007F])*")@(?:(?:[\da-z](?:[\da-z-]*[\da-z])?\.)+[\da-z](?:[\da-z-]*[\da-z])?|\[(?:(2(5[0-5]|[0-4]\d)|1\d\d|[1-9]?\d)\.){3}(?:(2(5[0-5]|[0-4]\d)|1\d\d|[1-9]?\d)|[\da-z-]*[\da-z]:(?:[\u0001-\u0008\u000B\u000C\u000E-\u001F\u0021-\u007F]|\\[\u0001-\u0009\u000B\u000C\u000E-\u007F])+)])/;

  return re.test(newMail);
}

function upper(str: string) {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

onMounted(async () => {
  const searchParam = new URLSearchParams(window.location.search);

  if (!searchParam.has("token")) {
    toast.toast({
      message: "Missing invite link",
      type: "error",
    });

    return;
  }

  const token = searchParam.get("token");

  try {
    const results = await useBackendFetch<Invite>(`/invite/${token}`);

    inviteData.value = results;

    useHeadSafe(
      {
        title: `Invite - ${results.token} :: K-Librarian`,
      },
      { head }
    );
  } catch (error) {
    if (error instanceof Error) {
      toast.toast({
        title: "Unknown error occured",
        message: error.message,
        type: "error",
      });
    } else {
      toast.toast({
        title: "Failed to fetch invite",
        message: String(error),
        type: "error",
      });
    }
  }

  if (validUserRef.value) {
    autoAnimate(validUserRef.value);
  }

  if (validPassRef.value) {
    autoAnimate(validPassRef.value);
  }
});

watch(
  () => username.value,
  (newUser) => {
    const errorValidation = [];
    if (newUser.length <= 0) {
      errorValidation.push("Username cannot be empty");
    }

    // only allow alphanumeric characters, dash, and underscore
    if (!/^[a-zA-Z0-9_-]+$/.test(newUser)) {
      errorValidation.push("Username can only contain alphanumeric characters, dash, and underscore");
    }
    validationUsername.value = errorValidation;
  }
);

watch(
  () => email.value,
  (newMail) => {
    if (newMail.length === 0) {
      validationEmail.value = ["Username/email cannot be empty"];
    } else if (isValidEmail(newMail)) {
      validationEmail.value = [];
    } else {
      validationEmail.value = ["Invalid email"];
    }
  }
);

watch(
  () => password.value,
  (newPass) => {
    validationPassword.value = newPass.length === 0 ? ["Password cannot be empty"] : [];
  }
);
</script>

<style scoped lang="postcss">
.server-width {
  @apply w-[80%] md:w-[60%] lg:w-[40%];
}
</style>

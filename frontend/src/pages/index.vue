<template>
  <main class="mx-auto my-auto flex h-screen w-full flex-col items-center justify-center">
    <i-mdi-book-outline class="mb-2 h-12 w-12" />
    <div class="font-variable text-xl variation-weight-bold">K-Librarian</div>
    <hr class="server-width my-4 border-gray-600 opacity-70 dark:border-gray-400" />
    <div class="server-width flex flex-col justify-start">
      <label for="invite-code" class="mb-2 text-sm">Have an invite code?</label>
      <div class="flex flex-row items-center">
        <input
          id="invite-code"
          ref="inputRef"
          v-model="inviteCode"
          class="form-input w-full transition disabled:cursor-not-allowed disabled:border-opacity-50 disabled:bg-gray-100 dark:bg-gray-800 disabled:dark:bg-gray-900"
          :placeholder="`/invite?token=... or just the token`"
          @keypress="interceptEnter"
        />
        <button
          class="aspect-square h-full w-auto px-2 py-2 transition hover:opacity-80 disabled:animate-pulse disabled:cursor-not-allowed"
          @click="redirectToInvite"
        >
          <i-mdi-arrow-right-thick class="mx-auto" />
        </button>
      </div>
    </div>
    <hr class="server-width my-4 border-gray-600 opacity-70 dark:border-gray-400" />
    <div class="mt-2 flex flex-row gap-2">
      <router-link to="/admin" class="transition hover:opacity-70 dark:hover:opacity-80">
        <i-mdi-login class="h-8 w-8" />
      </router-link>
      <dark-toggle />
    </div>
  </main>
  <footer-info />
</template>

<script setup lang="ts">
import useToast from "@/composables/use-toast";

const router = useRouter();
const toast = useToast();
const inputRef = ref<HTMLInputElement>();
const inviteCode = ref();

function tryToExtractInviteCode(inviteCode: string) {
  const uuidRe = /\w{8}(?:-\w{4}){3}-\w{12}/g;
  try {
    const url = new URL(inviteCode);

    const params = new URLSearchParams(url.search);
    const token = params.get("token");
    if (token && uuidRe.test(token)) {
      return token;
    }
  } catch {
    if (uuidRe.test(inviteCode)) {
      return inviteCode;
    }
  }

  return;
}

function redirectToInvite() {
  inputRef.value?.blur();

  // Extract the invite code from the input
  // It can either be a full URL or just the code
  const data = tryToExtractInviteCode(inviteCode.value);

  if (!data) {
    toast.toast({
      message: "Invalid invite code",
      type: "error",
    });

    return;
  }

  router.push({
    name: "/invite",
    query: {
      token: data,
    },
  });
}

function interceptEnter(event: KeyboardEvent) {
  if (event.key === "Enter") {
    event.preventDefault();

    if (inviteCode.value) {
      redirectToInvite();
    }
  }
}

onMounted(() => {
  useHeadSafe({
    title: "K-Librarian",
  });
});
</script>

<style scoped lang="postcss">
.server-width {
  @apply w-[80%] md:w-[60%] lg:w-[40%];
}
</style>

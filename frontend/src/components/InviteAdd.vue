<template>
  <div class="mb-4 flex flex-col gap-4 rounded-md bg-white px-2 py-2 dark:bg-gray-800">
    <div v-if="availableModes.length > 1" class="flex flex-col">
      <label class="font-variable mb-2 text-lg variation-weight-semibold">Mode</label>
      <select
        v-model="inviteMode"
        class="form-select mb-2 w-full rounded-md border-gray-300 dark:border-gray-600 dark:bg-gray-700"
      >
        <option v-for="mode in availableModes" :key="mode.mode" :value="mode.mode">{{ mode.label }}</option>
      </select>
    </div>
    <div class="flex flex-col">
      <label class="font-variable mb-1 text-lg variation-weight-semibold">Roles</label>
      <div class="flex flex-row items-center">
        <input v-model="roleAdmin" type="checkbox" class="form-checkbox mr-2 rounded-md" />
        <label>Administrator</label>
      </div>
      <slot v-if="inviteMode === 'komga'">
        <div class="flex flex-row items-center">
          <input v-model="roleFileDownload" type="checkbox" class="form-checkbox mr-2 rounded-md" />
          <label>File Download</label>
        </div>
        <div class="flex flex-row items-center">
          <input v-model="rolePageRead" type="checkbox" class="form-checkbox mr-2 rounded-md" />
          <label>Page Streaming (Reading)</label>
        </div>
        <div class="flex flex-row items-center">
          <input v-model="roleKoboSync" type="checkbox" class="form-checkbox mr-2 rounded-md" />
          <label>Kobo Sync</label>
        </div>
        <div class="flex flex-row items-center">
          <input v-model="roleKoReaderSync" type="checkbox" class="form-checkbox mr-2 rounded-md" />
          <label>KOReader Sync</label>
        </div>
      </slot>
    </div>
    <div class="flex flex-col">
      <label class="font-variable mb-1 text-lg variation-weight-semibold">Libraries</label>
      <div v-if="inviteMode === 'komga'" class="flex flex-row items-center">
        <input
          type="checkbox"
          class="form-checkbox mr-2 rounded-md"
          name="library"
          :checked="selectedLibraries.includes('all')"
          @click="selectedLibraries = selectedLibraries.includes('all') ? [] : ['all']"
        />
        <label>All</label>
      </div>
      <div v-for="library in computedLibraries" :key="library.value" class="flex flex-row items-center">
        <input
          type="checkbox"
          class="form-checkbox mr-2 rounded-md disabled:opacity-80"
          name="library"
          :checked="library.checked || (inviteMode === 'komga' ? selectedLibraries.includes('all') : roleAdmin)"
          :disabled="shouldDisableLibrary"
          @click="addToLibrary(library.value)"
        />
        <label>{{ library.label }}</label>
      </div>
    </div>
    <label class="font-variable text-lg variation-weight-semibold">Expiry</label>
    <vue-date-picker v-model="expiresAt" utc :dark="darkMode" :min-date="new Date()" />
  </div>
  <button
    class="font-variable mb-4 flex flex-row items-center justify-center border-2 border-green-500 bg-transparent px-2 py-2 text-sm text-green-500 transition variation-weight-[550] hover:bg-green-600 hover:text-white"
    @click="emitAdd"
  >
    <span class="text-center">Add</span>
  </button>
</template>

<script setup lang="ts">
import "@vuepic/vue-datepicker/dist/main.css";
import VueDatePicker from "@vuepic/vue-datepicker";
import useInviteConfig from "@/composables/use-invite-config";
import useDarkMode from "@/composables/use-dark-mode";
import useToast from "@/composables/use-toast";
import type { AddEmitKomga, AddEmitNavidrome } from "@/types/invites";

const emit = defineEmits<{
  (e: "add", data: AddEmitKomga | AddEmitNavidrome): void;
}>();

const darkMode = useDarkMode();
const inviteConfig = useInviteConfig();
const toasts = useToast();
const selectedLibraries = ref<string[]>(["all"]);
const selectedNavidromeLibraries = ref<number[]>([]);
const selectedLabels = ref<string[]>([]);
const selectedExcludeLabels = ref<string[]>([]);

const inviteMode = ref<"komga" | "navidrome">("komga");

// Roles
const expiresAt = ref<Date>();
const roleAdmin = ref(false);
const roleFileDownload = ref(true);
const rolePageRead = ref(true);
const roleKoboSync = ref(false);
const roleKoReaderSync = ref(false);

const shouldDisableLibrary = computed(() => {
  if (inviteMode.value === "navidrome") {
    return roleAdmin.value;
  }

  return selectedLibraries.value.includes("all");
});

const computedLibraries = computed(() => {
  if (inviteMode.value === "navidrome") {
    return (
      inviteConfig.inviteConfig?.navidrome.libraries.map((library) => ({
        label: library.name,
        value: library.id,
        checked: selectedNavidromeLibraries.value.includes(library.id),
      })) ?? []
    );
  }

  return (
    inviteConfig.inviteConfig?.komga.libraries
      .filter((library) => !library.unavailable)
      .map((library) => ({
        label: library.name,
        value: library.id,
        checked: selectedLibraries.value.includes(library.id),
      })) ?? []
  );
});

const availableModes = computed(() => {
  const activeModes = [
    {
      mode: "komga",
      label: "Komga",
    },
  ];
  if (inviteConfig.inviteConfig?.navidrome.active) {
    activeModes.push({
      mode: "navidrome",
      label: "Navidrome",
    });
  }

  return activeModes;
});

function addToLibrary(libraryId: string | number) {
  if (inviteMode.value === "navidrome") {
    if (selectedNavidromeLibraries.value.includes(libraryId as number)) {
      selectedNavidromeLibraries.value = selectedNavidromeLibraries.value.filter((id) => id !== libraryId);
    } else {
      selectedNavidromeLibraries.value.push(libraryId as number);
    }
    return;
  } else {
    if (selectedLibraries.value.includes(libraryId as string)) {
      selectedLibraries.value = selectedLibraries.value.filter((id) => id !== libraryId);
    } else {
      selectedLibraries.value.push(libraryId as string);
    }
  }
}

function emitAdd() {
  const unixTimestamp = expiresAt.value ? new Date(expiresAt.value).getTime() : -1;

  if (unixTimestamp !== -1 && unixTimestamp < Date.now()) {
    toasts.toast({
      message: "Expiry date cannot be in the past",
      type: "error",
      duration: 2500,
    });

    return;
  }

  if (inviteMode.value === "navidrome") {
    emit("add", {
      mode: "navidrome",
      libraries: selectedNavidromeLibraries.value,
      isAdmin: roleAdmin.value,
      expiresAt: unixTimestamp === -1 ? undefined : Math.floor(unixTimestamp / 1000),
    });
  } else {
    emit("add", {
      mode: "komga",
      libraries: selectedLibraries.value,
      labels: selectedLabels.value,
      excludeLabels: selectedExcludeLabels.value,
      roles: [
        "USER",
        roleAdmin.value ? "ADMIN" : "",
        roleFileDownload.value ? "FILE_DOWNLOAD" : "",
        rolePageRead.value ? "PAGE_STREAMING" : "",
        roleKoboSync.value ? "KOBO_SYNC" : "",
        roleKoReaderSync.value ? "KOREADER_SYNC" : "",
      ].filter((role) => role !== ""),
      expiresAt: unixTimestamp === -1 ? undefined : Math.floor(unixTimestamp / 1000),
    });
  }
}
</script>

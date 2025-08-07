<template>
  <div class="mb-4 flex flex-col gap-4 rounded-md bg-white px-2 py-2 dark:bg-gray-800">
    <div class="flex flex-col">
      <label class="font-variable text-lg variation-weight-semibold">Roles</label>
      <div class="flex flex-row items-center">
        <input v-model="roleAdmin" type="checkbox" class="form-checkbox mr-2 rounded-md" />
        <label>Administrator</label>
      </div>
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
    </div>
    <div class="flex flex-col">
      <label class="font-variable text-lg variation-weight-semibold">Libraries</label>
      <div class="flex flex-row items-center">
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
          :checked="library.checked || selectedLibraries.includes('all')"
          :disabled="selectedLibraries.includes('all')"
          @click="addToLibrary(library.value)"
        />
        <label>{{ library.label }}</label>
      </div>
    </div>
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

interface AddEmit {
  libraries: string[];
  labels: string[];
  excludeLabels: string[];
  roles: string[];
  expiresAt?: number | null;
}

const emit = defineEmits<{
  (e: "add", data: AddEmit): void;
}>();

const darkMode = useDarkMode();
const inviteConfig = useInviteConfig();
const toasts = useToast();
const selectedLibraries = ref<string[]>(["all"]);
const selectedLabels = ref<string[]>([]);
const selectedExcludeLabels = ref<string[]>([]);

// Roles
const expiresAt = ref<Date>();
const roleAdmin = ref(false);
const roleFileDownload = ref(true);
const rolePageRead = ref(true);
const roleKoboSync = ref(false);
const roleKoReaderSync = ref(false);

const computedLibraries = computed(() => {
  return (
    inviteConfig.inviteConfig?.libraries
      .filter((library) => !library.unavailable)
      .map((library) => ({
        label: library.name,
        value: library.id,
        checked: selectedLibraries.value.includes(library.id),
      })) ?? []
  );
});

function addToLibrary(libraryId: string) {
  if (selectedLibraries.value.includes(libraryId)) {
    selectedLibraries.value = selectedLibraries.value.filter((id) => id !== libraryId);
  } else {
    selectedLibraries.value.push(libraryId);
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

  emit("add", {
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
</script>

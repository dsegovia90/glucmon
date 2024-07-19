<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";

interface GlucmonConfig {
  nightscoutUrl: string;
  nightscoutApiToken: string;
  isMmmol: boolean;
}

const glucmonConfig = ref<GlucmonConfig>({
  nightscoutUrl: "",
  nightscoutApiToken: "String",
  isMmmol: false,
});

async function getGlucmonConfig() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  glucmonConfig.value = await invoke("get_glucmon_config");
}

async function updateGlucmonConfig() {
  console.log("SUBMITTING");
  glucmonConfig.value = await invoke("set_glucmon_config", {
    formConfigValues: glucmonConfig.value,
  });
  await appWindow.close();
}

onMounted(async () => {
  await getGlucmonConfig();
});

watch(glucmonConfig, () => {
  console.log(glucmonConfig.value);
});
</script>

<template>
  <form class="container grid gap-2" @submit.prevent="updateGlucmonConfig">
    <label
      for="url"
      class="block text-gray-700 dark:text-gray-300 text-sm font-bold mb-1"
    >
      Nightscout URL
    </label>
    <input
      type="text"
      id="url"
      class="block w-full border rounded px-2 py-1"
      v-model="glucmonConfig.nightscoutUrl"
      placeholder="Enter a url..."
    />
    <label
      for="apitoken"
      class="block text-gray-700 dark:text-gray-300 text-sm font-bold mb-1"
    >
      Nightscout API Token
    </label>
    <input
      type="text"
      id="apitoken"
      class="block border rounded px-2 py-1"
      v-model="glucmonConfig.nightscoutApiToken"
      placeholder="Enter api token..."
    />
    <div class="flex justify-center">
      <div class="flex gap-4 mb-8">
        <p class="text-sm font-medium text-gray-900 dark:text-gray-300">
          mg/dL
        </p>
        <label class="inline-flex items-center cursor-pointer">
          <input
            v-model="glucmonConfig.isMmmol"
            type="checkbox"
            id="units"
            class="sr-only peer"
          />
          <div
            class="relative w-11 h-6 bg-gray-400 rounded-full peer peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 dark:bg-gray-500 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600"
          />
        </label>
        <p class="text-sm font-medium text-gray-900 dark:text-gray-300">
          mmol/L
        </p>
      </div>
    </div>

    <button
      type="submit"
      class="bg-cyan-500 hover:bg-cyan-700 dark:bg-cyan-700 dark:hover:bg-cyan-500 text-white font-bold py-2 px-4 rounded-md focus:outline-none focus:shadow-outline transition-colors"
    >
      Save
    </button>
  </form>
</template>

<style scoped lang="scss">
input[type="text"] {
  @apply block w-full border rounded-md px-2 py-1 mb-8;
}
</style>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/tauri";
import { onMounted } from "vue";

onMounted(async () => {
  await invoke("get_glucose_data");
});
</script>

<template>
  <div class="container">
    <h1 class="underline">Welcome to Tauri!</h1>
  </div>
</template>

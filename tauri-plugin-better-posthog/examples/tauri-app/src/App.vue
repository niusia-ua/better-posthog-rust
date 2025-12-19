<script setup lang="ts">
import { usePostHog } from "./composables/";

const { posthog } = usePostHog();

function handleButtonClick() {
  posthog.capture("button_click", { label: "Press me!" });
}
</script>

<template>
  <h1>Hello World!</h1>
  <button @click="handleButtonClick">Press me!</button>
</template>

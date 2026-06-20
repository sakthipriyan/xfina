<script setup>
import { cn } from "@/lib/utils";

const props = defineProps({
  class: {
    type: [Boolean, null, String, Object, Array],
    required: false,
    skipCheck: true,
  },
});
</script>

<template>
  <tbody :class="cn('[&_tr:last-child]:border-0', props.class)">
    <slot />
  </tbody>
</template>

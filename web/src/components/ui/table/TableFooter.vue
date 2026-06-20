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
  <tfoot
    :class="
      cn('border-t bg-muted/50 font-medium [&>tr]:last:border-b-0', props.class)
    "
  >
    <slot />
  </tfoot>
</template>

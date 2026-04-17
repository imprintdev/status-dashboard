<script setup lang="ts">
defineProps<{ status: 'connecting' | 'open' | 'closed' | 'error' }>()
const labels = { connecting: 'Connecting', open: 'Live', closed: 'Disconnected', error: 'Error' }
</script>

<template>
  <div class="ws-indicator">
    <span :class="['ws-dot', status]"></span>
    {{ labels[status] }}
  </div>
</template>

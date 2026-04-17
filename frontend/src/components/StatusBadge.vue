<script setup lang="ts">
defineProps<{ status: string | null }>()
const label = (s: string | null) => s ? s.toUpperCase() : 'UNKNOWN'
const cls   = (s: string | null) => s ?? 'unknown'
</script>

<template>
  <span :class="['status-badge', cls(status)]">
    <span class="dot"></span>
    {{ label(status) }}
  </span>
</template>

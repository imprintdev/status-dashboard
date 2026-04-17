<script setup lang="ts">
import StatusBadge from './StatusBadge.vue'
import type { Service } from '../types'

const props = defineProps<{ service: Service }>()
const emit  = defineEmits<{ select: [id: string] }>()

const statusClass = () => {
  const s = props.service.latest_check?.status
  if (s === 'down')     return 'is-down'
  if (s === 'degraded') return 'is-degraded'
  if (s === 'up')       return 'is-up'
  return ''
}

const fmtMs = (ms: number | null | undefined) =>
  ms != null ? `${ms}ms` : '—'

const fmtType = (t: string) => t.replace(/_/g, ' ')
</script>

<template>
  <div :class="['service-card', statusClass()]" @click="emit('select', service.id)">
    <div class="card-header">
      <span class="card-name">{{ service.name }}</span>
      <span class="type-tag">{{ fmtType(service.service_type) }}</span>
    </div>
    <div style="margin-bottom: 10px">
      <StatusBadge :status="service.latest_check?.status ?? null" />
    </div>
    <div class="card-meta">
      <span>⏱ {{ fmtMs(service.latest_check?.response_ms) }}</span>
      <span v-if="!service.enabled" style="color: var(--color-degraded)">Disabled</span>
    </div>
  </div>
</template>

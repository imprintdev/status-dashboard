<script setup lang="ts">
import type { Service } from '../types'

defineProps<{ service: Service }>()

function statusLabel(service: Service): string {
  const s = service.latest_check?.status
  if (!s) return 'UNKNOWN'
  return s.toUpperCase()
}

function statusClass(service: Service): string {
  return service.latest_check?.status ?? 'unknown'
}

function formatMs(ms: number | null): string {
  if (ms === null) return ''
  return ms >= 1000 ? `${(ms / 1000).toFixed(1)}s` : `${ms}ms`
}
</script>

<template>
  <div class="tv-slice" :class="statusClass(service)">
    <div class="tv-slice-name">{{ service.name }}</div>
    <div class="tv-slice-status">{{ statusLabel(service) }}</div>
    <div v-if="service.latest_check?.response_ms != null" class="tv-slice-ms">
      {{ formatMs(service.latest_check.response_ms) }}
    </div>
  </div>
</template>

<style scoped>
.tv-slice {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 16px 12px;
  border-bottom: 2px solid rgba(0, 0, 0, 0.12);
  min-height: 0;
  position: relative;
}

.tv-slice:last-child {
  border-bottom: none;
}

.tv-slice.up       { background: #16a34a; }
.tv-slice.degraded { background: #d97706; }
.tv-slice.down     { background: #dc2626; }
.tv-slice.unknown  { background: #4b5563; }

.tv-slice-name {
  font-size: clamp(14px, 2vw, 28px);
  font-weight: 700;
  color: #000;
  text-align: center;
  line-height: 1.2;
  word-break: break-word;
}

.tv-slice-status {
  font-size: clamp(20px, 3vw, 48px);
  font-weight: 900;
  color: #000;
  letter-spacing: 2px;
  margin-top: 4px;
}

.tv-slice-ms {
  font-size: clamp(10px, 1.2vw, 18px);
  font-weight: 600;
  color: rgba(0, 0, 0, 0.55);
  margin-top: 4px;
  font-variant-numeric: tabular-nums;
}

.tv-slice.down .tv-slice-status {
  animation: pulse-text 1.5s infinite;
}

@keyframes pulse-text {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.6; }
}
</style>

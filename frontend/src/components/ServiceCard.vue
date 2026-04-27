<script setup lang="ts">
import StatusBadge from './StatusBadge.vue'
import ChartWidget from './ChartWidget.vue'
import type { Service } from '../types'
import { serviceStats } from '../utils/serviceDetail'

const props = defineProps<{ service: Service }>()
const emit  = defineEmits<{ select: [id: string] }>()

const isChart = () => props.service.service_type === 'chart_query'
const isHttpBody = () => props.service.service_type === 'http_body'

function cardBody(): string | null {
  const detail = props.service.latest_check?.detail
  if (!detail) return null
  const d = typeof detail === 'string' ? (() => { try { return JSON.parse(detail) } catch { return null } })() : detail
  const body = (d as Record<string, unknown>)?.body
  return typeof body === 'string' ? body : null
}

const statusClass = () => {
  const s = props.service.latest_check?.status
  if (s === 'down')     return 'is-down'
  if (s === 'degraded') return 'is-degraded'
  if (s === 'up')       return 'is-up'
  return ''
}

const fmtMs = (ms: number | null | undefined) => ms != null ? `${ms}ms` : '—'
const fmtType = (t: string) => t.replace(/_/g, ' ')
</script>

<template>
  <!-- Chart card — taller, wider -->
  <div v-if="isChart()" :class="['service-card', 'chart-card', statusClass()]" @click="emit('select', service.id)">
    <div class="card-header">
      <span class="card-name">{{ service.name }}</span>
      <span class="type-tag">{{ fmtType(service.service_type) }}</span>
    </div>
    <ChartWidget :service="service" />
    <div class="card-meta" style="margin-top: 6px">
      <StatusBadge :status="service.latest_check?.status ?? null" />
      <span>⏱ {{ fmtMs(service.latest_check?.response_ms) }}</span>
      <span v-if="!service.enabled" style="color: var(--color-degraded)">Disabled</span>
    </div>
  </div>

  <!-- Standard card -->
  <div v-else :class="['service-card', statusClass()]" @click="emit('select', service.id)">
    <div class="card-header">
      <span class="card-name">{{ service.name }}</span>
      <span class="type-tag">{{ fmtType(service.service_type) }}</span>
    </div>
    <div style="margin-bottom: 10px">
      <StatusBadge :status="service.latest_check?.status ?? null" />
    </div>
    <div v-if="serviceStats(service).length > 0" class="card-stats">
      <span v-for="stat in serviceStats(service)" :key="stat.label" class="stat-item">
        <span class="stat-label">{{ stat.label }}</span>
        <span class="stat-value">{{ stat.value }}</span>
      </span>
    </div>
    <pre v-if="isHttpBody() && cardBody()" class="card-body-pre">{{ cardBody() }}</pre>
    <div v-if="!service.enabled" class="card-meta">
      <span style="color: var(--color-degraded)">Disabled</span>
    </div>
  </div>
</template>

<style scoped>
.card-body-pre {
  font-size: 11px;
  font-family: 'SFMono-Regular', Consolas, monospace;
  color: var(--text-muted);
  background: var(--bg-hover);
  border-radius: var(--radius-sm);
  padding: 8px 10px;
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.6;
  margin: 6px 0 0;
  max-height: 120px;
  overflow-y: auto;
}
</style>

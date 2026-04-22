<script setup lang="ts">
import type { Service } from '../types'
import { serviceStats } from '../utils/serviceDetail'
import ChartWidget from './ChartWidget.vue'

defineProps<{ service: Service }>()

function statusLabel(service: Service): string {
  const s = service.latest_check?.status
  if (!s) return 'UNKNOWN'
  return s.toUpperCase()
}

function statusClass(service: Service): string {
  return service.latest_check?.status ?? 'unknown'
}
</script>

<template>
  <!-- Chart slice -->
  <div v-if="service.service_type === 'chart_query'" class="tv-slice tv-slice-chart" :class="statusClass(service)">
    <div class="tv-slice-chart-header">
      <span class="tv-slice-name">{{ service.name }}</span>
      <span class="tv-slice-chart-status">{{ statusLabel(service) }}</span>
    </div>
    <div class="tv-slice-chart-body">
      <ChartWidget :service="service" />
    </div>
  </div>

  <!-- Standard slice -->
  <div v-else class="tv-slice" :class="statusClass(service)">
    <span class="tv-slice-watermark">{{ statusLabel(service) }}</span>
    <div class="tv-slice-name">{{ service.name }}</div>
    <div v-if="serviceStats(service).length > 0" class="tv-slice-stats">
      <span v-for="stat in serviceStats(service)" :key="stat.label" class="tv-slice-stat">
        <span class="tv-slice-stat-val">{{ stat.value }}</span>
        <span class="tv-slice-stat-label">{{ stat.label }}</span>
      </span>
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

/* Chart slice overrides */
.tv-slice-chart {
  flex: 3;
  align-items: stretch;
  justify-content: flex-start;
  padding: 0;
}

.tv-slice-chart.up       { background: #052e16; }
.tv-slice-chart.degraded { background: #1c1003; }
.tv-slice-chart.down     { background: #1c0202; }
.tv-slice-chart.unknown  { background: #111827; }

.tv-slice-chart-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-bottom: 2px solid rgba(255, 255, 255, 0.08);
  flex-shrink: 0;
}

.tv-slice-chart-status {
  font-size: clamp(9px, 0.9vw, 13px);
  font-weight: 700;
  color: rgba(255, 255, 255, 0.4);
  letter-spacing: 1px;
}

.tv-slice-chart-body {
  flex: 1;
  min-height: 0;
  padding: 10px;
}

/* Standard slice text */
.tv-slice-name {
  font-size: clamp(14px, 2vw, 28px);
  font-weight: 700;
  color: #000;
  text-align: center;
  line-height: 1.2;
  word-break: break-word;
}

.tv-slice-chart .tv-slice-name {
  font-size: clamp(12px, 1.2vw, 18px);
  color: #e2e8f0;
}

.tv-slice-watermark {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: clamp(40px, 6vw, 100px);
  font-weight: 900;
  letter-spacing: 3px;
  color: #fff;
  opacity: 0.2;
  pointer-events: none;
  user-select: none;
}

.tv-slice-stats {
  display: flex;
  gap: clamp(8px, 1.5vw, 24px);
  margin-top: 8px;
  flex-wrap: wrap;
  justify-content: center;
}

.tv-slice-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.tv-slice-stat-val {
  font-size: clamp(18px, 2.5vw, 40px);
  font-weight: 900;
  color: #000;
  font-variant-numeric: tabular-nums;
}

.tv-slice-stat-label {
  font-size: clamp(9px, 1vw, 14px);
  font-weight: 600;
  color: rgba(0, 0, 0, 0.55);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.tv-slice.down .tv-slice-status {
  animation: pulse-text 1.5s infinite;
}

@keyframes pulse-text {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.6; }
}
</style>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed, nextTick } from 'vue'
import { Chart, registerables } from 'chart.js'
import type { Service } from '../types'

Chart.register(...registerables)

const props = defineProps<{ service: Service }>()

const canvas = ref<HTMLCanvasElement | null>(null)
let chart: Chart | null = null

interface ChartRow { label: string; value: number; color?: string }

const DEFAULT_COLORS = [
  '#4f8ef7', '#f7674f', '#4ff79e', '#f7c84f', '#c84ff7',
  '#4ff7f0', '#f74fb0', '#a3f74f', '#f7a34f', '#4f6af7',
]

const chartType = computed(() => {
  const t = props.service.config.chart_type as string | undefined
  if (t === 'bar' || t === 'line') return t
  return 'pie'
})

const title = computed(() => props.service.config.title as string | undefined)
const xLabel = computed(() => props.service.config.x_label as string | undefined)
const yLabel = computed(() => props.service.config.y_label as string | undefined)

function parseRows(detail: unknown): ChartRow[] {
  if (!detail || typeof detail !== 'object') return []
  const d = detail as Record<string, unknown>
  const rows = d.rows
  if (!Array.isArray(rows)) return []
  return rows.filter((r): r is ChartRow =>
    r && typeof r === 'object' && typeof r.label === 'string' && typeof r.value === 'number'
  )
}

function buildChart(rows: ChartRow[]) {
  if (!canvas.value) return
  if (chart) { chart.destroy(); chart = null }

  const labels = rows.map(r => r.label)
  const values = rows.map(r => r.value)
  const colors = rows.map((r, i) => r.color ?? DEFAULT_COLORS[i % DEFAULT_COLORS.length])

  const type = chartType.value

  chart = new Chart(canvas.value, {
    type,
    data: {
      labels,
      datasets: [{
        data: values,
        backgroundColor: type === 'line' ? colors[0] + '33' : colors,
        borderColor: colors,
        borderWidth: type === 'line' ? 2 : 1,
        fill: type === 'line',
        tension: type === 'line' ? 0.3 : undefined,
        pointBackgroundColor: type === 'line' ? colors : undefined,
      }],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: { duration: 400 },
      plugins: {
        legend: {
          display: type === 'pie',
          labels: { color: '#ccc', boxWidth: 12, font: { size: 11 } },
        },
        title: {
          display: false,
        },
        tooltip: {
          callbacks: {
            label: (ctx) => ` ${ctx.formattedValue}`,
          },
        },
      },
      scales: type !== 'pie' ? {
        x: {
          title: { display: !!xLabel.value, text: xLabel.value, color: '#aaa', font: { size: 11 } },
          ticks: { color: '#aaa', font: { size: 11 } },
          grid: { color: '#333' },
        },
        y: {
          title: { display: !!yLabel.value, text: yLabel.value, color: '#aaa', font: { size: 11 } },
          ticks: { color: '#aaa', font: { size: 11 } },
          grid: { color: '#333' },
          beginAtZero: true,
        },
      } : undefined,
    },
  })
}

async function refresh() {
  await nextTick()
  const rows = parseRows(props.service.latest_check?.detail)
  if (rows.length > 0) buildChart(rows)
}

onMounted(refresh)
onBeforeUnmount(() => { chart?.destroy() })

watch(() => props.service.latest_check?.detail, refresh, { flush: 'post' })
watch(chartType, refresh, { flush: 'post' })
</script>

<template>
  <div class="chart-widget">
    <div v-if="title" class="chart-widget-title">{{ title }}</div>
    <div v-if="!service.latest_check?.detail || parseRows(service.latest_check.detail).length === 0" class="chart-empty">
      No data yet
    </div>
    <div v-else class="chart-canvas-wrap">
      <canvas ref="canvas" />
    </div>
  </div>
</template>

<style scoped>
.chart-widget {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 6px;
}
.chart-widget-title {
  font-size: 12px;
  color: var(--color-text-muted, #aaa);
  text-align: center;
}
.chart-canvas-wrap {
  flex: 1;
  min-height: 0;
  position: relative;
}
.chart-canvas-wrap canvas {
  position: absolute;
  inset: 0;
  width: 100% !important;
  height: 100% !important;
}
.chart-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  color: #666;
}
</style>

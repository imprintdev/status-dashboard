<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useServicesStore } from '../stores/services'
import { useSystemsStore } from '../stores/systems'
import TvSystemColumn from './TvSystemColumn.vue'
import TvServiceSlice from './TvServiceSlice.vue'

const servicesStore = useServicesStore()
const systemsStore  = useSystemsStore()

const systems   = computed(() => systemsStore.list)
const ungrouped = computed(() => servicesStore.ungrouped)

const now = ref(new Date())
let timer: ReturnType<typeof setInterval>
onMounted(() => { timer = setInterval(() => { now.value = new Date() }, 1000) })
onUnmounted(() => clearInterval(timer))

const timeStr = computed(() =>
  now.value.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
)
</script>

<template>
  <div class="tv-root">
    <div class="tv-grid">
      <TvSystemColumn
        v-for="sys in systems"
        :key="sys.id"
        :system="sys"
      />

      <!-- Ungrouped services as a final column -->
      <div v-if="ungrouped.length > 0" class="tv-column tv-ungrouped">
        <div class="tv-column-header tv-ungrouped-header">
          <span class="tv-column-title">Other</span>
        </div>
        <div class="tv-column-body">
          <TvServiceSlice
            v-for="svc in ungrouped"
            :key="svc.id"
            :service="svc"
          />
        </div>
      </div>
    </div>

    <div class="tv-footer">
      <span class="tv-footer-title">Status Dashboard</span>
      <span class="tv-footer-time">{{ timeStr }}</span>
    </div>
  </div>
</template>

<style scoped>
.tv-root {
  position: fixed;
  inset: 0;
  display: flex;
  flex-direction: column;
  background: #111;
  overflow: hidden;
}

.tv-grid {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  min-height: 0;
}

.tv-column {
  display: flex;
  flex-direction: column;
  height: 100%;
  border-right: 3px solid rgba(0, 0, 0, 0.3);
  overflow: hidden;
}

.tv-column:last-child {
  border-right: none;
}

.tv-column-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 14px 10px 12px;
  border-bottom: 3px solid rgba(0, 0, 0, 0.2);
  flex-shrink: 0;
}

.tv-ungrouped-header {
  background: #374151;
}

.tv-column-title {
  font-size: clamp(16px, 2vw, 32px);
  font-weight: 800;
  color: #000;
  text-align: center;
  line-height: 1.1;
}

.tv-column-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.tv-footer {
  height: 40px;
  flex-shrink: 0;
  background: #000;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
}

.tv-footer-title {
  font-size: 13px;
  font-weight: 700;
  color: #555;
  letter-spacing: 1px;
  text-transform: uppercase;
}

.tv-footer-time {
  font-size: 14px;
  font-weight: 600;
  color: #666;
  font-variant-numeric: tabular-nums;
  font-family: 'SFMono-Regular', Consolas, monospace;
}
</style>

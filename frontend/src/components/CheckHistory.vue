<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useCheckResultsStore } from '../stores/checkResults'
import { useServicesStore } from '../stores/services'
import { checkStats } from '../utils/serviceDetail'

const props = defineProps<{ serviceId: string }>()
const store = useCheckResultsStore()
const servicesStore = useServicesStore()

onMounted(() => store.fetchForService(props.serviceId))

const results = computed(() => store.byService[props.serviceId] ?? [])
const loading = computed(() => store.loading[props.serviceId])
const serviceType = computed(() => servicesStore.items[props.serviceId]?.service_type ?? '')

const fmtTime = (iso: string) =>
  new Date(iso).toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit', second: '2-digit' })
</script>

<template>
  <div class="check-list">
    <div v-if="loading && results.length === 0" class="loading">Loading…</div>
    <div v-else-if="results.length === 0" class="loading">No checks recorded yet.</div>
    <div v-for="r in results" :key="r.id" class="check-row">
      <span :class="['check-dot', r.status]"></span>
      <span class="check-time">{{ fmtTime(r.checked_at) }}</span>
      <span v-if="r.error_message" class="check-err" :title="r.error_message">{{ r.error_message }}</span>
      <span class="check-stats">
        <span
          v-for="stat in checkStats(serviceType, r.detail, r.response_ms)"
          :key="stat.label"
          class="check-stat"
        >
          <span class="check-stat-label">{{ stat.label }}</span>
          <span class="check-stat-value">{{ stat.value }}</span>
        </span>
      </span>
    </div>
  </div>
</template>

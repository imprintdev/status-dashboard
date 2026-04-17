<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useIncidentsStore } from '../stores/incidents'

const props = defineProps<{ serviceId: string }>()
const store = useIncidentsStore()

onMounted(() => store.fetchForService(props.serviceId))

const incidents = computed(() => store.byService[props.serviceId] ?? [])
const loading   = computed(() => store.loading[props.serviceId])

const fmtTime = (iso: string) =>
  new Date(iso).toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })

async function resolve(incidentId: string) {
  await store.resolve(props.serviceId, incidentId)
}
</script>

<template>
  <div class="incident-list">
    <div v-if="loading && incidents.length === 0" class="loading">Loading…</div>
    <div v-else-if="incidents.length === 0" class="loading">No incidents recorded.</div>
    <div v-for="inc in incidents" :key="inc.id" class="incident-row">
      <div class="incident-header">
        <span :class="['incident-status', inc.status]">{{ inc.status }}</span>
        <button
          v-if="inc.status === 'open'"
          class="btn btn-sm"
          @click="resolve(inc.id)"
        >Resolve</button>
      </div>
      <div class="incident-times">
        Started {{ fmtTime(inc.started_at) }}
        <span v-if="inc.resolved_at"> · Resolved {{ fmtTime(inc.resolved_at) }}</span>
      </div>
      <div v-if="inc.notes" class="incident-notes">{{ inc.notes }}</div>
    </div>
  </div>
</template>

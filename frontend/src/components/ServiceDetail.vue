<script setup lang="ts">
import { ref, computed } from 'vue'
import { useServicesStore } from '../stores/services'
import StatusBadge from './StatusBadge.vue'
import CheckHistory from './CheckHistory.vue'
import IncidentList from './IncidentList.vue'

const props = defineProps<{ serviceId: string }>()
const emit  = defineEmits<{ close: []; edit: [id: string] }>()

const store   = useServicesStore()
const service = computed(() => store.items[props.serviceId])
const tab     = ref<'checks' | 'incidents'>('checks')
</script>

<template>
  <div class="detail-panel" v-if="service">
    <div class="detail-header">
      <div>
        <div class="detail-title">{{ service.name }}</div>
        <div class="detail-meta">
          <span>{{ service.service_type.replace(/_/g, ' ') }}</span>
          <span>every {{ service.interval_secs }}s</span>
          <StatusBadge :status="service.latest_check?.status ?? null" />
        </div>
      </div>
      <div class="detail-actions">
        <button class="icon-btn" title="Edit service" @click="emit('edit', service.id)">✏️</button>
        <button class="icon-btn" title="Close" @click="emit('close')">✕</button>
      </div>
    </div>

    <div class="detail-tabs">
      <button :class="['tab-btn', { active: tab === 'checks' }]"    @click="tab = 'checks'">Check History</button>
      <button :class="['tab-btn', { active: tab === 'incidents' }]" @click="tab = 'incidents'">Incidents</button>
    </div>

    <div class="detail-body">
      <CheckHistory  v-if="tab === 'checks'"    :service-id="serviceId" />
      <IncidentList  v-if="tab === 'incidents'" :service-id="serviceId" />
    </div>
  </div>
</template>

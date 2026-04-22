<script setup lang="ts">
import { ref, computed } from 'vue'
import { useServicesStore } from '../stores/services'
import StatusBadge from './StatusBadge.vue'
import type { System } from '../types'
import { checkStats } from '../utils/serviceDetail'

const props = defineProps<{ system: System }>()
const emit  = defineEmits<{
  selectService: [id: string]
  editSystem: [id: string]
}>()

const open = ref(true)
const servicesStore = useServicesStore()
const services = computed(() => servicesStore.bySystem(props.system.id))

const fmtType = (t: string) => t.replace(/_/g, ' ')
</script>

<template>
  <div class="system-card">
    <div class="system-card-header" @click="open = !open">
      <span :class="['system-collapse-icon', { open }]">▶</span>
      <span class="system-card-title">{{ system.name }}</span>
      <StatusBadge :status="system.health" />
      <span class="system-card-meta">{{ system.service_count }} check{{ system.service_count === 1 ? '' : 's' }}</span>
      <button
        class="icon-btn"
        title="Edit system"
        @click.stop="emit('editSystem', system.id)"
      >✏️</button>
    </div>

    <div v-if="open" class="system-services">
      <div v-if="services.length === 0" class="loading" style="padding: 10px 16px; font-size: 12px">
        No checks in this system yet.
      </div>
      <div
        v-for="svc in services"
        :key="svc.id"
        class="system-service-row"
        @click="emit('selectService', svc.id)"
      >
        <span :class="['check-dot', svc.latest_check?.status ?? 'unknown']"></span>
        <span class="system-service-name">{{ svc.name }}</span>
        <span class="system-service-type">{{ fmtType(svc.service_type) }}</span>
        <span class="system-service-stats">
          <span
            v-for="stat in checkStats(svc.service_type, svc.latest_check?.detail, svc.latest_check?.response_ms ?? null)"
            :key="stat.label"
            class="system-service-stat"
          >{{ stat.label }}: {{ stat.value }}</span>
        </span>
      </div>
    </div>
  </div>
</template>

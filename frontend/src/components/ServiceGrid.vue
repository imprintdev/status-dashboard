<script setup lang="ts">
import { computed } from 'vue'
import { useServicesStore } from '../stores/services'
import ServiceCard from './ServiceCard.vue'

const emit = defineEmits<{ select: [id: string] }>()
const store = useServicesStore()
const services = computed(() => store.list)
</script>

<template>
  <div class="service-grid">
    <template v-if="store.loading && services.length === 0">
      <div class="loading">Loading services…</div>
    </template>
    <template v-else-if="store.error">
      <div class="error-msg">{{ store.error }}</div>
    </template>
    <template v-else-if="services.length === 0">
      <div class="empty-state">
        <p>No services configured.</p>
        <p style="font-size: 13px">Click <strong>Add Service</strong> to get started.</p>
      </div>
    </template>
    <ServiceCard
      v-for="svc in services"
      :key="svc.id"
      :service="svc"
      @select="emit('select', $event)"
    />
  </div>
</template>

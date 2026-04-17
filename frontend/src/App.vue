<script setup lang="ts">
import { ref, computed } from 'vue'
import { useServicesStore } from './stores/services'
import { useWebSocket } from './composables/useWebSocket'
import WsIndicator   from './components/WsIndicator.vue'
import ServiceGrid   from './components/ServiceGrid.vue'
import ServiceModal  from './components/ServiceModal.vue'
import ServiceDetail from './components/ServiceDetail.vue'

const { status } = useWebSocket()
const store = useServicesStore()

const showModal    = ref(false)
const editingId    = ref<string | null>(null)
const selectedId   = ref<string | null>(null)

const editingService = computed(() =>
  editingId.value ? store.items[editingId.value] : undefined
)

function openAddModal() {
  editingId.value = null
  showModal.value = true
}

function openEditModal(id: string) {
  editingId.value = id
  showModal.value = true
}

function selectService(id: string) {
  selectedId.value = id === selectedId.value ? null : id
}
</script>

<template>
  <nav class="nav">
    <span class="nav-title">Status Dashboard</span>
    <WsIndicator :status="status" />
    <button class="btn btn-primary" @click="openAddModal">+ Add Service</button>
  </nav>

  <ServiceGrid @select="selectService" />

  <ServiceDetail
    v-if="selectedId"
    :service-id="selectedId"
    @close="selectedId = null"
    @edit="openEditModal"
  />

  <ServiceModal
    v-if="showModal"
    :service="editingService"
    @close="showModal = false"
  />
</template>

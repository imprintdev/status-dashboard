<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useServicesStore } from './stores/services'
import { useSystemsStore } from './stores/systems'
import { useWebSocket } from './composables/useWebSocket'
import WsIndicator   from './components/WsIndicator.vue'
import ServiceGrid   from './components/ServiceGrid.vue'
import ServiceModal  from './components/ServiceModal.vue'
import ServiceDetail from './components/ServiceDetail.vue'
import SystemModal   from './components/SystemModal.vue'
import TvDashboard   from './components/TvDashboard.vue'

const { status } = useWebSocket()
const servicesStore = useServicesStore()
const systemsStore  = useSystemsStore()

const showServiceModal = ref(false)
const editingServiceId = ref<string | null>(null)

const showSystemModal  = ref(false)
const editingSystemId  = ref<string | null>(null)

const selectedId = ref<string | null>(null)

const tvMode = ref(new URLSearchParams(window.location.search).has('tv'))

function onKey(e: KeyboardEvent) {
  if (e.key === 't' || e.key === 'T') {
    if (document.activeElement?.tagName === 'INPUT' || document.activeElement?.tagName === 'TEXTAREA') return
    tvMode.value = !tvMode.value
  }
}
onMounted(() => window.addEventListener('keydown', onKey))
onUnmounted(() => window.removeEventListener('keydown', onKey))

const editingService = computed(() =>
  editingServiceId.value ? servicesStore.items[editingServiceId.value] : undefined
)
const editingSystem = computed(() =>
  editingSystemId.value ? systemsStore.items[editingSystemId.value] : undefined
)

function openAddService() {
  editingServiceId.value = null
  showServiceModal.value = true
}

function openEditService(id: string) {
  editingServiceId.value = id
  showServiceModal.value = true
}

function openAddSystem() {
  editingSystemId.value = null
  showSystemModal.value = true
}

function openEditSystem(id: string) {
  editingSystemId.value = id
  showSystemModal.value = true
}

function selectService(id: string) {
  selectedId.value = id === selectedId.value ? null : id
}
</script>

<template>
  <TvDashboard v-if="tvMode" />

  <template v-else>
    <nav class="nav">
      <span class="nav-title">Status Dashboard</span>
      <WsIndicator :status="status" />
      <button class="btn" @click="openAddSystem">+ System</button>
      <button class="btn btn-primary" @click="openAddService">+ Service</button>
    </nav>

    <ServiceGrid
      @select="selectService"
      @editSystem="openEditSystem"
    />

    <ServiceDetail
      v-if="selectedId"
      :service-id="selectedId"
      @close="selectedId = null"
      @edit="openEditService"
    />

    <ServiceModal
      v-if="showServiceModal"
      :service="editingService"
      @close="showServiceModal = false"
    />

    <SystemModal
      v-if="showSystemModal"
      :system="editingSystem"
      @close="showSystemModal = false"
    />
  </template>
</template>

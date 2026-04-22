<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { api } from '../api/http'
import { useServicesStore } from '../stores/services'
import { useSystemsStore } from '../stores/systems'
import type { Service } from '../types'

const props = defineProps<{ service?: Service }>()
const emit  = defineEmits<{ close: [] }>()

const store = useServicesStore()
const systemsStore = useSystemsStore()
const isEdit = computed(() => !!props.service)

// Form state
const name         = ref(props.service?.name ?? '')
const serviceType  = ref(props.service?.service_type ?? 'http')
const configText   = ref(props.service ? JSON.stringify(props.service.config, null, 2) : defaultConfig('http'))
const intervalSecs = ref(props.service?.interval_secs ?? 60)
const enabled      = ref(props.service?.enabled ?? true)
const systemIds    = ref<string[]>(props.service?.system_ids ?? [])
const error        = ref('')
const saving       = ref(false)
const deleting     = ref(false)

const SERVICE_TYPES = [
  { value: 'http',        label: 'HTTP Endpoint' },
  { value: 'database',    label: 'Database' },
  { value: 'aws_billing', label: 'AWS Billing' },
  { value: 'php_site',    label: 'PHP Site' },
  { value: 'preflight',   label: 'Preflight Script' },
  { value: 'sql_query',   label: 'SQL Query' },
  { value: 'chart_query', label: 'Chart Query' },
]

function defaultConfig(type: string): string {
  const configs: Record<string, object> = {
    http: {
      url: 'https://example.com',
      method: 'GET',
      expected_status: 200,
      timeout_ms: 10000,
      degraded_ms: 2000,
    },
    database: {
      connection_string: 'sqlite:./my.db',
      probe_query: 'SELECT 1',
      degraded_ms: 1000,
    },
    aws_billing: {
      region: 'us-east-1',
      access_key_id: 'AKIA...',
      secret_access_key: '...',
      threshold_usd: 500,
      degraded_pct: 0.8,
    },
    php_site: {
      url: 'https://example.com',
      fpm_status_url: 'http://localhost/fpm-status',
      expected_content: null,
      timeout_ms: 10000,
      degraded_ms: 3000,
    },
    preflight: {
      command: '/usr/local/bin/check.sh',
      args: [],
      expected_exit_code: 0,
      timeout_ms: 30000,
      degraded_ms: 10000,
    },
    sql_query: {
      connection_string: 'postgresql://user:pass@host/db',
      query: 'SELECT COUNT(*) FROM order_logs WHERE status > 400 AND date(created_at) = CURRENT_DATE',
      down_threshold: { gt: 1000 },
      degraded_threshold: { gt: 500 },
      timeout_ms: 10000,
    },
    chart_query: {
      connection_string: 'sqlite:./my.db',
      query: 'SELECT status AS label, COUNT(*) AS value FROM requests GROUP BY status',
      chart_type: 'pie',
      title: 'Requests by Status',
      timeout_ms: 10000,
    },
  }
  return JSON.stringify(configs[type] ?? {}, null, 2)
}

watch(serviceType, (t) => {
  if (!isEdit.value) configText.value = defaultConfig(t)
})

const configHints: Record<string, string> = {
  http:        'url, method, expected_status, timeout_ms, degraded_ms, headers',
  database:    'connection_string (sqlite: or postgresql://), probe_query, degraded_ms',
  aws_billing: 'region, access_key_id, secret_access_key, threshold_usd, degraded_pct',
  php_site:    'url, fpm_status_url?, expected_content?, timeout_ms, degraded_ms',
  preflight:   'command, args[], expected_exit_code, timeout_ms, degraded_ms',
  sql_query:   'connection_string (sqlite: or postgresql://), query, down_threshold { gt/lt/gte/lte/eq/neq }, degraded_threshold?, timeout_ms',
  chart_query: 'connection_string (sqlite: or postgresql://), query (returns label+value rows), chart_type (pie/bar/line), title?, x_label?, y_label?, timeout_ms',
}

async function submit() {
  error.value = ''
  let config: unknown
  try {
    config = JSON.parse(configText.value)
  } catch {
    error.value = 'Config must be valid JSON.'
    return
  }

  saving.value = true
  try {
    if (isEdit.value && props.service) {
      await api.updateService(props.service.id, {
        name: name.value,
        config,
        interval_secs: intervalSecs.value,
        enabled: enabled.value,
        system_ids: systemIds.value,
      })
      const updated = await api.fetchService(props.service.id)
      store.upsert(updated)
    } else {
      await api.createService({
        name: name.value,
        service_type: serviceType.value,
        config,
        interval_secs: intervalSecs.value,
        system_ids: systemIds.value,
      })
      await store.fetchAll()
    }
    emit('close')
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Save failed'
  } finally {
    saving.value = false
  }
}

async function deleteService() {
  if (!props.service) return
  if (!confirm(`Delete "${props.service.name}"? This cannot be undone.`)) return
  deleting.value = true
  try {
    await api.deleteService(props.service.id)
    store.remove(props.service.id)
    emit('close')
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Delete failed'
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal">
      <div class="modal-header">
        <span class="modal-title">{{ isEdit ? 'Edit Service' : 'Add Service' }}</span>
        <button class="icon-btn" @click="emit('close')">✕</button>
      </div>

      <form class="modal-body" @submit.prevent="submit">
        <div class="form-group">
          <label for="svc-name">Name</label>
          <input id="svc-name" type="text" v-model="name" placeholder="My API" required />
        </div>

        <div class="form-group">
          <label for="svc-type">Service Type</label>
          <select id="svc-type" v-model="serviceType" :disabled="isEdit">
            <option v-for="t in SERVICE_TYPES" :key="t.value" :value="t.value">{{ t.label }}</option>
          </select>
        </div>

        <div class="form-group">
          <label for="svc-config">Config (JSON)</label>
          <textarea id="svc-config" v-model="configText" spellcheck="false"></textarea>
          <span class="form-hint">{{ configHints[serviceType] }}</span>
        </div>

        <div class="form-group">
          <label for="svc-interval">Poll Interval (seconds)</label>
          <input id="svc-interval" type="number" v-model.number="intervalSecs" min="5" max="86400" />
        </div>

        <div class="form-group">
          <label>Systems (optional)</label>
          <div class="checkbox-list">
            <span v-if="systemsStore.list.length === 0" class="form-hint">No systems created yet</span>
            <label v-for="sys in systemsStore.list" :key="sys.id" class="checkbox-item">
              <input type="checkbox" :value="sys.id" v-model="systemIds" />
              {{ sys.name }}
            </label>
          </div>
        </div>

        <div v-if="isEdit" class="toggle-row">
          <label for="svc-enabled">Enabled</label>
          <input id="svc-enabled" type="checkbox" v-model="enabled" />
        </div>

        <div v-if="error" class="form-error">{{ error }}</div>
      </form>

      <div class="modal-footer">
        <div>
          <button
            v-if="isEdit"
            type="button"
            class="btn btn-danger btn-sm"
            :disabled="deleting"
            @click="deleteService"
          >{{ deleting ? 'Deleting…' : 'Delete' }}</button>
        </div>
        <div class="modal-footer-right">
          <button type="button" class="btn" @click="emit('close')">Cancel</button>
          <button type="submit" class="btn btn-primary" :disabled="saving" @click="submit">
            {{ saving ? 'Saving…' : (isEdit ? 'Save Changes' : 'Add Service') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

import { defineStore } from 'pinia'
import { api } from '../api/http'
import type { CheckResult, WsMessage } from '../types'

const MAX_RESULTS = 100

export const useCheckResultsStore = defineStore('checkResults', {
  state: () => ({
    byService: {} as Record<string, CheckResult[]>,
    loading: {} as Record<string, boolean>,
  }),

  actions: {
    async fetchForService(id: string) {
      if (this.loading[id]) return
      this.loading[id] = true
      try {
        const results = await api.fetchChecks(id, { limit: MAX_RESULTS })
        this.byService[id] = results
      } finally {
        this.loading[id] = false
      }
    },

    prepend(serviceId: string, msg: Extract<WsMessage, { type: 'check_completed' }>) {
      const existing = this.byService[serviceId]
      if (!existing) return
      const result: CheckResult = {
        id: msg.check_id,
        service_id: serviceId,
        checked_at: msg.checked_at,
        status: msg.status as 'up' | 'degraded' | 'down',
        response_ms: msg.response_ms,
        detail: null,
        error_message: msg.error_message,
      }
      this.byService[serviceId] = [result, ...existing].slice(0, MAX_RESULTS)
    },
  },
})

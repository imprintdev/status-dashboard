import { defineStore } from 'pinia'
import { api } from '../api/http'
import type { Service, WsMessage } from '../types'

export const useServicesStore = defineStore('services', {
  state: () => ({
    items: {} as Record<string, Service>,
    loading: false,
    error: null as string | null,
  }),

  getters: {
    list: (state) => Object.values(state.items).sort((a, b) => a.created_at.localeCompare(b.created_at)),
  },

  actions: {
    async fetchAll() {
      this.loading = true
      this.error = null
      try {
        const services = await api.fetchServices()
        const map: Record<string, Service> = {}
        for (const s of services) map[s.id] = s
        this.items = map
      } catch (e) {
        this.error = e instanceof Error ? e.message : 'Failed to load services'
      } finally {
        this.loading = false
      }
    },

    upsert(service: Service) {
      this.items[service.id] = service
    },

    remove(id: string) {
      delete this.items[id]
    },

    applyCheckUpdate(msg: Extract<WsMessage, { type: 'check_completed' }>) {
      const svc = this.items[msg.service_id]
      if (!svc) return
      svc.latest_check = {
        id: msg.check_id,
        checked_at: msg.checked_at,
        status: msg.status as 'up' | 'degraded' | 'down',
        response_ms: msg.response_ms,
        error_message: msg.error_message,
      }
    },

    applyServiceUpdate(msg: Extract<WsMessage, { type: 'service_updated' }>) {
      const svc = this.items[msg.service_id]
      if (!svc) return
      Object.assign(svc, msg.fields)
    },
  },
})

import { defineStore } from 'pinia'
import { api } from '../api/http'
import type { Incident, WsMessage } from '../types'

export const useIncidentsStore = defineStore('incidents', {
  state: () => ({
    byService: {} as Record<string, Incident[]>,
    loading: {} as Record<string, boolean>,
  }),

  actions: {
    async fetchForService(id: string) {
      if (this.loading[id]) return
      this.loading[id] = true
      try {
        this.byService[id] = await api.fetchIncidents(id)
      } finally {
        this.loading[id] = false
      }
    },

    applyOpened(msg: Extract<WsMessage, { type: 'incident_opened' }>) {
      const list = this.byService[msg.service_id]
      if (!list) return
      const incident: Incident = {
        id: msg.incident_id,
        service_id: msg.service_id,
        started_at: msg.started_at,
        resolved_at: null,
        status: 'open',
        trigger_status: msg.trigger_status,
        notes: null,
      }
      this.byService[msg.service_id] = [incident, ...list]
    },

    applyResolved(msg: Extract<WsMessage, { type: 'incident_resolved' }>) {
      const list = this.byService[msg.service_id]
      if (!list) return
      const incident = list.find(i => i.id === msg.incident_id)
      if (incident) {
        incident.resolved_at = msg.resolved_at
        incident.status = 'resolved'
      }
    },

    async resolve(serviceId: string, incidentId: string, notes?: string) {
      const updated = await api.resolveIncident(serviceId, incidentId, notes)
      const list = this.byService[serviceId]
      if (list) {
        const idx = list.findIndex(i => i.id === incidentId)
        if (idx !== -1) list[idx] = updated
      }
    },
  },
})

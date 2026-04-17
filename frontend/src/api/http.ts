import type { CheckResult, Incident, Service } from '../types'

const BASE = (import.meta.env.VITE_API_BASE as string | undefined) ?? ''

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json', ...init?.headers },
    ...init,
  })
  if (!res.ok) {
    const body = await res.text().catch(() => '')
    throw new Error(`HTTP ${res.status}: ${body}`)
  }
  if (res.status === 204) return undefined as T
  return res.json()
}

export const api = {
  fetchServices: () => request<Service[]>('/api/services'),

  fetchService: (id: string) => request<Service>(`/api/services/${id}`),

  createService: (body: { name: string; service_type: string; config: unknown; interval_secs?: number }) =>
    request<{ id: string; name: string; created_at: string }>('/api/services', {
      method: 'POST',
      body: JSON.stringify(body),
    }),

  updateService: (id: string, body: { name?: string; config?: unknown; interval_secs?: number; enabled?: boolean }) =>
    request<{ id: string; updated_at: string }>(`/api/services/${id}`, {
      method: 'PUT',
      body: JSON.stringify(body),
    }),

  deleteService: (id: string) =>
    request<void>(`/api/services/${id}`, { method: 'DELETE' }),

  fetchChecks: (id: string, params?: { limit?: number; before_id?: string }) => {
    const q = new URLSearchParams()
    if (params?.limit)     q.set('limit',     String(params.limit))
    if (params?.before_id) q.set('before_id', params.before_id)
    const qs = q.toString() ? `?${q}` : ''
    return request<CheckResult[]>(`/api/services/${id}/checks${qs}`)
  },

  fetchUptime: (id: string, window: '24h' | '7d' | '30d' | '90d' = '7d') =>
    request<{ window: string; uptime_pct: number | null; total_checks: number; up_checks: number }>(
      `/api/services/${id}/uptime?window=${window}`
    ),

  fetchIncidents: (id: string) =>
    request<Incident[]>(`/api/services/${id}/incidents`),

  resolveIncident: (serviceId: string, incidentId: string, notes?: string) =>
    request<Incident>(`/api/services/${serviceId}/incidents/${incidentId}`, {
      method: 'PATCH',
      body: JSON.stringify({ notes: notes ?? null }),
    }),
}

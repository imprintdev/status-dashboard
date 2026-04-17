export interface LatestCheck {
  id: string
  checked_at: string
  status: 'up' | 'degraded' | 'down'
  response_ms: number | null
  error_message: string | null
}

export interface Service {
  id: string
  name: string
  service_type: string
  config: Record<string, unknown>
  interval_secs: number
  enabled: boolean
  created_at: string
  updated_at: string
  latest_check: LatestCheck | null
}

export interface CheckResult {
  id: string
  service_id: string
  checked_at: string
  status: 'up' | 'degraded' | 'down'
  response_ms: number | null
  detail: string | null
  error_message: string | null
}

export interface Incident {
  id: string
  service_id: string
  started_at: string
  resolved_at: string | null
  status: 'open' | 'resolved'
  trigger_status: string
  notes: string | null
}

export type WsMessage =
  | { type: 'check_completed'; service_id: string; check_id: string; checked_at: string; status: string; response_ms: number | null; detail: unknown; error_message: string | null }
  | { type: 'incident_opened'; incident_id: string; service_id: string; started_at: string; trigger_status: string }
  | { type: 'incident_resolved'; incident_id: string; service_id: string; resolved_at: string }
  | { type: 'service_updated'; service_id: string; fields: Record<string, unknown> }
  | { type: 'ping'; ts: string }

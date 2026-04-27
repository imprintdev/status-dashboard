import type { Service } from '../types'

export interface Stat {
  label: string
  value: string
}

function fmtMs(ms: number | null | undefined): string {
  if (ms == null) return '—'
  return ms >= 1000 ? `${(ms / 1000).toFixed(1)}s` : `${ms}ms`
}

function fmtUsd(v: unknown): string {
  if (typeof v !== 'number') return '—'
  return `$${v.toFixed(2)}`
}

function parseDetail(raw: unknown): Record<string, unknown> | null {
  if (!raw) return null
  if (typeof raw === 'string') {
    try { return JSON.parse(raw) } catch { return null }
  }
  if (typeof raw === 'object') return raw as Record<string, unknown>
  return null
}

export function checkStats(serviceType: string, rawDetail: unknown, ms: number | null): Stat[] {
  const detail = parseDetail(rawDetail)

  switch (serviceType) {
    case 'http':
    case 'http_body':
      return [
        ...(detail?.http_status != null ? [{ label: 'HTTP', value: String(detail.http_status) }] : []),
        { label: 'time', value: fmtMs(ms) },
      ]

    case 'database':
      return [
        ...(detail?.driver != null ? [{ label: 'driver', value: String(detail.driver) }] : []),
        { label: 'time', value: fmtMs(ms) },
      ]

    case 'aws_billing':
      return [
        { label: 'spent', value: fmtUsd(detail?.cost_usd) },
        { label: 'limit', value: fmtUsd(detail?.threshold_usd) },
      ]

    case 'php_site':
      return [
        ...(detail?.http_status != null ? [{ label: 'HTTP', value: String(detail.http_status) }] : []),
        { label: 'FPM', value: detail?.fpm_ok === true ? 'ok' : detail?.fpm_error != null ? String(detail.fpm_error).slice(0, 20) : '—' },
        { label: 'time', value: fmtMs(ms) },
      ]

    case 'preflight':
      return [
        ...(detail?.exit_code != null ? [{ label: 'exit', value: String(detail.exit_code) }] : []),
        ...(detail?.stdout && String(detail.stdout).trim() ? [{ label: 'out', value: String(detail.stdout).trim().split('\n')[0].slice(0, 30) }] : []),
        { label: 'time', value: fmtMs(ms) },
      ]

    case 'sql_query':
    case 'chart_query': {
      if (!detail) return []
      return Object.entries(detail)
        .filter(([, v]) => typeof v === 'number' || typeof v === 'string')
        .map(([k, v]) => ({ label: k, value: String(v) }))
    }

    default:
      return ms != null ? [{ label: 'time', value: fmtMs(ms) }] : []
  }
}

export function serviceStats(service: Service): Stat[] {
  const check = service.latest_check
  return checkStats(service.service_type, check?.detail, check?.response_ms ?? null)
}

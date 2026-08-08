import { describe, expect, it } from 'vitest'
import { ClientSurfaceAvailabilityStateV1 } from '../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { recoveryClientBootstrap } from '../gateway/clientBootstrap'
import {
  systemControlAvailableSurfaceCount,
  systemControlSurfaceRows,
  systemControlSurfaceStateLabel,
} from './systemControlPresentation'

describe('system control presentation', () => {
  it('projects recovery surface availability and labels', () => {
    const bootstrap = recoveryClientBootstrap()
    const rows = systemControlSurfaceRows(bootstrap, ['system-control'])

    expect(rows).toHaveLength(13)
    expect(systemControlAvailableSurfaceCount(bootstrap)).toBe(1)
    expect(rows.find((row) => row.routeId === 'settings')?.available).toBe(true)
    expect(systemControlSurfaceStateLabel('dashboard', ClientSurfaceAvailabilityStateV1.BLOCKED, false)).toBe('Blocked')
  })
})

import * as React from 'react'
import { Box } from '@v/web-platform'
import { usePromise } from '@v/web-utils'
import { API } from '../api'
import { FrameTitle, MetaField } from '../parts'

export function StatusView() {
  const [status] = usePromise(() => API.get_status(), [])

  return (
    <>
      <FrameTitle>
        Status
      </FrameTitle>

      <Box>
        <MetaField title="arhiv id">
          {status?.db_status.arhiv_id}
        </MetaField>

        <MetaField title="is prime">
          {status?.db_status.is_prime.toString()}
        </MetaField>

        {status?.db_status.is_prime && (
          <MetaField title="root dir">
            {status.root_dir}
          </MetaField>
        )}

        <MetaField title="rev">
          {status?.db_status.db_rev}
        </MetaField>

        <MetaField title="committed documents">
          {status?.committed_documents}
        </MetaField>

        <MetaField title="staged documents">
          {status?.staged_documents}
        </MetaField>

        <MetaField title="last update time">
          {status?.last_update_time}
        </MetaField>

        <MetaField title="last sync time">
          {status?.db_status.last_sync_time}
        </MetaField>

        <MetaField title="debug mode">
          {status?.debug_mode.toString()}
        </MetaField>
      </Box>
    </>
  )
}

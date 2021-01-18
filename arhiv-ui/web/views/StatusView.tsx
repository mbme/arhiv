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
          {status?.arhivId}
        </MetaField>

        <MetaField title="is prime">
          {status?.isPrime.toString()}
        </MetaField>

        {status?.isPrime && (
          <MetaField title="root dir">
            {status.rootDir}
          </MetaField>
        )}

        <MetaField title="rev">
          {status?.rev}
        </MetaField>

        <MetaField title="committed documents">
          {status?.committedDocuments}
        </MetaField>

        <MetaField title="staged documents">
          {status?.stagedDocuments}
        </MetaField>

        <MetaField title="last update time">
          {status?.lastUpdateTime}
        </MetaField>

        <MetaField title="debug mode">
          {status?.debugMode.toString()}
        </MetaField>
      </Box>
    </>
  )
}

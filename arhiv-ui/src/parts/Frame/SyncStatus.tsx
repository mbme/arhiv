import * as React from 'react'
import { Box } from '@v/web-platform'
import { API } from '../../api'

export function SyncStatus() {
  const [isSyncRequired, setSyncRequired] = React.useState(false)

  React.useEffect(() => {
    const update = () => {
      void API.is_sync_required().then(setSyncRequired)
    }

    update()

    const intervalId = setInterval(update, 30000)

    return () => {
      clearInterval(intervalId)
    }
  }, [])

  return (
    <Box color="red">
      {isSyncRequired && 'Sync required'}
    </Box>
  )
}

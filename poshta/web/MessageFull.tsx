import * as React from 'react'
import {
  Box,
  Row,
  Spacer,
} from '@v/web-platform'
import { IGmailMessage } from './gmail'
import { ChronoFormatter } from '@v/chrono'

const dateFormat = new ChronoFormatter('HH:mm DD.MM.YYYY')

function getHeaderValue(message: IGmailMessage, name: string): string | undefined {
  const header = message.payload.headers.find(item => item.name === name)
  if (!header) {
    return undefined
  }

  return header.value
}

interface IProps {
  message: IGmailMessage
  focused?: boolean
}

export function MessageFull({ message }: IProps ) {
  return (
    <Box
      mb="large"
      p="medium"
    >
      <Row alignX="left">
        {message.labelIds.map(label => (
          <Box key={label} bgColor="yellow" mr="fine">
            {label}
          </Box>
        ))}
        <Spacer />
        <Box>
          {dateFormat.format(new Date(getHeaderValue(message, 'Date') || ''))}
        </Box>
      </Row>

      <h3>{getHeaderValue(message, 'Subject')}</h3>

      <Box
        dangerouslySetInnerHTML={{ __html: message.snippet }}
        mb="large"
      />

      <Box>
        <b>From: </b> {getHeaderValue(message, 'From')}
      </Box>
      <Box>
        <b>To: </b> {getHeaderValue(message, 'To')}
      </Box>
    </Box>
  )
}

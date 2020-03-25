import { IQueryParam, stringifyQueryParams } from '@v/web-utils'

const BASE_URL = 'https://www.googleapis.com/gmail/v1/users/me'

interface IGmailProfile {
  emailAddress: string,
  messagesTotal: number,
  threadsTotal: number,
}

interface IGmailMessageHeader {
  name: string,
  value: string,
}

interface IGmailMessagePayload {
  body: IGmailMessagePayloadBody,
  filename?: string,
  headers: IGmailMessageHeader[],
  mimeType: string,
  partId: string,
  parts?: IGmailMessagePayload[],
}

interface IGmailMessagePayloadBody {
  attachmentId?: string,
  data: string, // base64
  size: number,
}

// https://developers.google.com/gmail/api/v1/reference/users/messages#resource
export interface IGmailMessage {
  id: string,
  internalDate: number,
  labelIds: string[],
  payload: IGmailMessagePayload,
  sizeEstimate: number,
  snippet: string,
  threadId: string,
}

interface IGmailMessageRef {
  id: string,
  threadId: string,
}

interface IGmailMessageList {
  messages: IGmailMessageRef[],
  nextPageToken: string,
  resultSizeEstimate: number,
}

export class Gmail {
  constructor(private _token: string) {}

  private async _get(path: string, params: IQueryParam[] = []) {
    const response = await fetch(BASE_URL + path + stringifyQueryParams(params), {
      method: 'GET',
      headers: {
        Authorization: `Bearer ${this._token}`,
      },
    })

    if (!response.ok) {
      throw new Error(`GET ${path} failed: ${response.status}`)
    }

    return response.json()
  }

  // https://developers.google.com/gmail/api/v1/reference/users/getProfile
  getProfile(): Promise<IGmailProfile> {
    return this._get('/profile')
  }

  // https://developers.google.com/gmail/api/v1/reference/users/messages/list
  listMessages(q?: string, maxResults = 100) {
    let page = -1
    let total = -1
    let nextPageToken: string | undefined = undefined

    const params: IQueryParam[] = [
      {
        name: 'q',
        value: q,
      },
      {
        name: 'maxResults',
        value: maxResults.toString(),
      },
      {
        name: 'includeSpamTrash',
        value: 'true',
      }
    ]

    return {
      get page() {
        return page
      },

      get total() {
        return total
      },

      loadNextPage: async (): Promise<IGmailMessage[]> => {
        page += 1

        const result: IGmailMessageList = await this._get('/messages', [
          ...params,
          {
            name: 'nextPageToken',
            value: nextPageToken?.toString(),
          },
        ])

        total = result.resultSizeEstimate
        nextPageToken = result.nextPageToken

        return this.batchGetMessages(result.messages.map(ref => ref.id))
      },
    }
  }

  getMessage(id: string): Promise<IGmailMessage> {
    return this._get(`/messages/${id}`)
  }

  // https://developers.google.com/gmail/api/guides/batch#example
  async batchGetMessages(ids: string[]): Promise<IGmailMessage[]> {
    const boundary = 'batch-get-message'

    const bodyItems = ids.map(id => [
      `--${boundary}\n`,
      'Content-Type: application/http\n',
      '\n',
      `GET /gmail/v1/users/me/messages/${id}\n`,
      '\n',
    ].join(''))

    bodyItems.push(`--${boundary}--`)

    const response = await fetch('https://www.googleapis.com/batch/gmail/v1', {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${this._token}`,
        'Content-Type': `multipart/mixed; boundary="${boundary}"`,
      },
      body: bodyItems.join('\n'),
    })

    if (!response.ok) {
      throw new Error(`batch get_messages failed: ${response.status}`)
    }
    const responseBoundary = response.headers.get('content-type')?.split('boundary=')[1]

    if (!responseBoundary) {
      throw new Error('batch get_messages failed: response boundary is missing')
    }

    const items = (await response.text()).split('--' + responseBoundary)
    items.shift() // drop first empty chunk
    items.pop() // drop last empty chunk

    return items.map((chunk) => {
      const i1 = chunk.indexOf('\r\n\r\n') // skip part headers
      const i2 = chunk.indexOf('\r\n\r\n', i1 + 4) // skip response headers

      return JSON.parse(chunk.substring(i2 + 4))
    })
  }
}

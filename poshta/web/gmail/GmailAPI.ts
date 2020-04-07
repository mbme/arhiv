import { IQueryParam, stringifyQueryParams } from '@v/web-utils'
import {
  IGmailProfile,
  IGmailMessageList,
  IGmailMessage,
} from './types'
import { GmailMessage } from './GmailMessage'

const BASE_URL = 'https://www.googleapis.com/gmail/v1/users/me'

export class GmailAPI {
  constructor(private _token: string) {}

  private async _get<T = object>(path: string, params: IQueryParam[] = []): Promise<T> {
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

      loadNextPage: async (): Promise<GmailMessage[]> => {
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

  async getMessage(id: string): Promise<GmailMessage> {
    const data: IGmailMessage = await this._get(`/messages/${id}`)

    return new GmailMessage(data)
  }

  // https://developers.google.com/gmail/api/guides/batch#example
  async batchGetMessages(ids: string[]): Promise<GmailMessage[]> {
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

      const data = JSON.parse(chunk.substring(i2 + 4))

      return new GmailMessage(data)
    })
  }
}

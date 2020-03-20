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
  body: object,
  filename?: string,
  headers: IGmailMessageHeader[],
  mimeType: string,
  partId: string,
  parts: object[],
}

// https://developers.google.com/gmail/api/v1/reference/users/messages#resource
interface IGmailMessage {
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

        return Promise.all(result.messages.map(msgRef => this.getMessage(msgRef.id)))
      },
    }
  }

  getMessage(id: string): Promise<IGmailMessage> {
    return this._get(`/messages/${id}`)
  }
}

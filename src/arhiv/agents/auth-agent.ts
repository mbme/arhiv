import { createLogger } from '~/logger'
import { NetworkAgent } from './network-agent'
import { WebClientEvents } from '../events'

const log = createLogger('arhiv:auth-agent')

type State = 'unauthorized' | 'authorized'

export class AuthAgent {
  state: State = 'authorized'

  constructor(
    public events: WebClientEvents,
    public networkAgent: NetworkAgent,
  ) { }

  isAuthorized() {
    return this.state === 'authorized'
  }

  private _notify() {
    this.events.emit('authorized', this.isAuthorized())
    log.info(`-> ${this.state}`)
  }

  async authorize(password: string) {
    if (await this.networkAgent.authorize(password)) {
      this.state = 'authorized'
      this._notify()
    }
  }

  deauthorize() {
    document.cookie = 'token=0; path=/'
    this.state = 'unauthorized'
    this._notify()
  }

  onNetworkError = (statusCode: number) => {
    if (statusCode === 403) {
      this.state = 'unauthorized'
      this._notify()
    }
  }

  start() {
    this.events.on('network-error', this.onNetworkError)
    log.debug('started')
  }

  stop() {
    this.events.off('network-error', this.onNetworkError)
    log.debug('stopped')
  }
}

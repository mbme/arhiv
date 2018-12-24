import NetworkAgent from './network-agent'
import { WebClientEvents } from './events'

type State = 'unauthorized' | 'authorized'

export default class AuthAgent {
  state: State = 'authorized'

  constructor(
    public events: WebClientEvents,
    public networkAgent: NetworkAgent
  ) { }

  _notify() {
    this.events.emit('authorized', this.state === 'authorized')
  }

  async authorize(password: string) {
    if (await this.networkAgent.authorize(password)) {
      this.state = 'authorized'
      this._notify()
    }
  }

  async deauthorize() {
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
  }

  stop() {
    this.events.off('network-error', this.onNetworkError)
  }
}

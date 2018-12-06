import http from 'http'
import urlParser from 'url'
import zlib from 'zlib'
import log from '../logger'
import { isString, isObject } from '../utils'
import { gzip, pipePromise } from '../utils/node'
import { Stream } from 'stream'
import { IContext, Next, HttpMethod, parseHttpMethod, IHeaders, IRequest, IResponse } from './types'
import bodyParserMiddleware from './body-parser-middleware'

type Middleware = (context: IContext, next: Next) => Promise<void> | void
type RequestHandler = (context: IContext) => Promise<void> | void
type PathTest = ((path: string) => boolean) | string

interface IRoute {
  test: (context: IContext) => boolean
  cb: RequestHandler
}

export default class Server {
  _server: http.Server | undefined

  _middlewares: Middleware[] = [bodyParserMiddleware]
  _routes: IRoute[] = []

  use(cb: Middleware) {
    this._middlewares.push(cb)
  }

  addRoute(method: HttpMethod, pathTest: PathTest, cb: RequestHandler) {
    this._routes.push({
      test({ req }: IContext) {
        if (req.method !== method.toString()) return false
        if (isString(pathTest)) return req.url.pathname === pathTest
        return pathTest(req.url.pathname!)
      },
      cb,
    })
  }

  get(pathTest: PathTest, cb: RequestHandler) {
    this.addRoute(HttpMethod.GET, pathTest, cb)
  }

  post(pathTest: PathTest, cb: RequestHandler) {
    this.addRoute(HttpMethod.POST, pathTest, cb)
  }

  async _runMiddlewares(context: IContext, pos: number) {
    const middleware = this._middlewares[pos]

    if (!middleware) return // no more middlewares, stop evaluation

    const next = () => this._runMiddlewares(context, pos + 1)
    await Promise.resolve(middleware(context, next))
  }

  start(port: number) {
    // router middleware
    this._middlewares.push(async (context) => {
      const route = this._routes.find((item) => item.test(context))

      if (route) {
        await Promise.resolve(route.cb(context))
      } else {
        context.res.statusCode = 404
      }
    })

    this._server = http.createServer(async (httpReq: http.IncomingMessage, httpRes: http.ServerResponse) => {
      const hrstart = process.hrtime()

      const isGzipSupported = /\bgzip\b/.test((httpReq.headers as IHeaders)['accept-encoding'])

      const req: IRequest = {
        url: urlParser.parse(httpReq.url!, true),
        method: parseHttpMethod(httpReq.method!)!,
        headers: httpReq.headers as IHeaders,
      }

      const res: IResponse = {
        statusCode: 200,
        headers: {},
        body: undefined,
      }

      try {
        await this._runMiddlewares({ req, res, httpReq, httpRes }, 0)
      } catch (e) {
        log.warn('failed to handle request', e)
        res.statusCode = 400
        res.body = { error: e.toString() }
      }

      httpRes.statusCode = res.statusCode

      for (const [header, value] of Object.entries(res.headers)) {
        httpRes.setHeader(header, value)
      }

      if (isGzipSupported) {
        httpRes.setHeader('Content-Encoding', 'gzip')
      }

      if (res.body instanceof Stream.Readable) {
        const stream = isGzipSupported ? res.body.pipe(zlib.createGzip()) : res.body
        await pipePromise(stream, httpRes)
      } else if (isObject(res.body)) {
        httpRes.setHeader('Content-Type', 'application/json')
        const str = JSON.stringify(res.body)
        httpRes.end(isGzipSupported ? await gzip(str) : str)
      } else {
        httpRes.end(res.body)
      }

      const hrend = process.hrtime(hrstart)
      const ms = (hrend[0] * 1000) + Math.round(hrend[1] / 1000000)
      log.debug(
        '%s %s %d %s - %dms',
        httpReq.method!.padEnd(4),
        httpReq.url,
        httpRes.statusCode,
        httpRes.statusMessage || 'OK',
        ms
      )
    })

    return new Promise((resolve) => this._server!.listen(port, resolve))
  }

  stop() {
    return new Promise((resolve) => this._server!.close(resolve))
  }
}

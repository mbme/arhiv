/// <reference lib="WebWorker" />

import {
  createLogger,
  configureLogger,
} from '~/logger'

configureLogger({
  minLogLevel: 'INFO',
})

const log = createLogger('serviceWorker', 'gray')

type Scope = ServiceWorkerGlobalScope & typeof globalThis
const scope = self as any as Scope // eslint-disable-line no-restricted-globals

const staticAssets = [
  './index.html',
  './favicon-16x16.png',
  './logo.svg',
  './serviceWorker.js',
  './bundle.js',
]

const CACHE_KEY = 'static-cache'
const openCache = () => caches.open(CACHE_KEY)

const cacheStaticAssets = async () => {
  const cache = await openCache()
  await cache.addAll(staticAssets)
}

const cleanupCaches = async () => {
  const cacheKeys = await caches.keys()

  await Promise.all(cacheKeys.map(async (cacheKey) => {
    if (cacheKey !== CACHE_KEY) {
      await caches.delete(cacheKey)
    }
  }))
}

async function networkFirst(req: Request): Promise<Response> {
  log.debug('fetching ', req.url)

  if (req.url.includes('/api/')) {
    return fetch(req).catch((e) => {
      log.warn(`failed to fetch ${req.url}: ${e}`)

      return new Response(null, {
        'status': 503,
        'statusText': 'Service Unavailable',
      })
    })
  }

  const cache = await openCache()
  const [
    cachedResponse,
    html5Fallback,
  ] = await Promise.all([
    cache.match(req),
    cache.match('./index.html'),
  ])

  if (!html5Fallback) { // unreachable
    throw new Error("html5 fallback isn't cached")
  }

  try {
    const res = await fetch(req)
    if (!res.ok) {
      throw new Error(`server responded with status code ${res.status}`)
    }

    await cache.put(req, res.clone())

    return res
  } catch (error) {
    log.warn('Failed to fetch cached resource', error)

    return cachedResponse || html5Fallback
  }
}

scope.addEventListener('install', (e) => {
  const freshnessPromise = scope.skipWaiting() // make sure we use new serviceWorker immediately

  e.waitUntil(freshnessPromise.then(cacheStaticAssets))
})

scope.addEventListener('activate', (e) => {
  e.waitUntil(cleanupCaches())
})

scope.addEventListener('fetch', (e) => {
  if (!e.request.url.startsWith('http')) { // ignore chrome-extension:// requests
    return
  }

  e.respondWith(networkFirst(e.request))
})

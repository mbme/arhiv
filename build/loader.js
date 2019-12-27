import path from 'path'
import {
  URL,
  pathToFileURL,
} from 'url'

const baseURL = pathToFileURL(process.cwd()).href

export async function resolve(specifier, parentModuleURL = baseURL, defaultResolver) {
  if (specifier.startsWith('~')) {
    const relativePath = specifier.substring(1)
    return defaultResolver(path.join(baseURL, 'tsdist/src', relativePath), parentModuleURL)
  }

  return defaultResolver(specifier, parentModuleURL)
}

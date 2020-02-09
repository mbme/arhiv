import { IDocumentModule } from './types'
import { DocumentModule } from './document'

export { IDocumentModule }

const Modules: IDocumentModule[] = []

export function useModule(module: IDocumentModule) {
  if (Modules.find(item => item.type === module.type)) {
    throw new Error(`Can't register duplicate module for type ${module.type}`)
  }

  Modules.push(module)
}

export function getModule(type: string): IDocumentModule {
  for (const Module of Modules) {
    if (Module.type === type) {
      return Module
    }
  }

  return DocumentModule
}

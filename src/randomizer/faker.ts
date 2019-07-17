import path from 'path'
import {
  IDocument,
  IAttachment,
  DocumentType,
} from '~/isodb/types'
import { generateRandomId } from '~/isodb/utils'
import {
  createArray,
  nowS,
  IDict,
} from '~/utils'
import { sha256File } from '~/utils/node'
import {
  readText,
  listFiles,
} from '~/utils/fs'
import { createLink } from '~/markup-parser/utils'
import {
  randomInt,
  shuffle,
  randomArrValue,
} from './index'
import createTextGenerator, { ITextGenerator } from './text-generator'

async function getFakeNote(generator: ITextGenerator, images: IDict): Promise<IDocument> {
  const name = generator.sentence(1, 8)

  const refs = new Set<string>()

  const data = createArray(
    randomInt(1, 7), // paragraphs
    () => {
      const sentences = createArray(
        randomInt(1, 7), // sentences
        () => generator.sentence(),
      )

      if (Math.random() < 0.34) {
        const hash = randomArrValue(Object.keys(images))
        refs.add(hash)

        const link = createLink(hash, path.basename(images[hash]))
        sentences.push(` ${link} `)
      }

      return shuffle(sentences).join(' ')
    },
  ).join('\n\n')

  const now = nowS()

  return {
    _id: generateRandomId(),
    _refs: [],
    _attachmentRefs: Array.from(refs),
    _rev: 1,
    _type: DocumentType.Note,
    _createdTs: now,
    _updatedTs: now,
    name: name.substring(0, name.length - 1),
    data,
  }
}

async function listImages(basePath: string): Promise<IDict> {
  const files = await listFiles(basePath)
  const images = files.filter((name) => name.match(/\.(jpg|jpeg)$/i))

  const result: IDict = {}

  await Promise.all(images.map(async (name) => {
    const filePath = path.join(basePath, name)
    const hash = await sha256File(filePath)
    result[hash] = filePath
  }))

  return result
}

function createAttachments(ids: string[]) {
  return ids.map(id => {
    const attachment: IAttachment = {
      _id: id,
      _rev: 1,
      _type: '',
      _size: 999,
    }

    return attachment
  })
}

export async function getFakeNotes(count: number) {
  const resourcesPath = path.join(process.env.BASE_DIR!, 'resources')
  const images = await listImages(resourcesPath)
  const text = await readText(path.join(resourcesPath, 'text.txt'))
  const generator = createTextGenerator(text)

  return {
    documents: await Promise.all(createArray(count, () => getFakeNote(generator, images))),
    attachments: createAttachments(Object.keys(images)),
    attachedFiles: images,
  }
}

import path from 'path'
import { NewAttachment } from '~/isodb/types'
import {
  generateRandomId,
  createDocument,
} from '~/isodb/utils'
import {
  DocumentType,
  INote,
} from '~/arhiv/types'
import {
  createArray,
  IDict,
  nowS,
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

async function getFakeNote(generator: ITextGenerator, images: IDict): Promise<INote> {
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

  return {
    ...createDocument(generateRandomId(), DocumentType.Note),
    _attachmentRefs: Array.from(refs),
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

function createAttachments(ids: string[]): NewAttachment[] {
  return ids.map(_id => ({ _id, _rev: 0, _createdTs: nowS() }))
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

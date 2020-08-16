import {
  createArray,
} from '@v/utils'
import {
  randomInt,
  shuffle,
  pickRandomItem,
} from '@v/utils/src/random'
import {
  ITextGenerator,
  createTextGenerator,
} from '@v/utils/src/random/text-generator'
import {
  createLink,
} from '../web/markup-parser'

interface IData {
  name: string
  data: string
}

function getFakeNote(generator: ITextGenerator, attachmentIds: string[]): IData {
  const name = generator.sentence(1, 8)

  const data = createArray(
    randomInt(1, 7), // paragraphs
    () => {
      const sentences = createArray(
        randomInt(1, 7), // sentences
        () => generator.sentence(),
      )

      if (Math.random() < 0.34 && attachmentIds.length) {
        const attachmentId = pickRandomItem(attachmentIds)

        const link = createLink(attachmentId, attachmentId)
        sentences.push(` ${link} `)
      }

      return shuffle(sentences).join(' ')
    },
  ).join('\n\n')

  return {
    name: name.substring(0, name.length - 1),
    data,
  }
}

interface IProps {
  text: string
  attachmentIds: string[]
  count: number
}

function main(propsStr: string) {
  const {
    text,
    attachmentIds,
    count,
  }: IProps = JSON.parse(propsStr)

  const generator = createTextGenerator(text)

  const result = createArray(count, () => getFakeNote(generator, attachmentIds))

  process.stdout.write(JSON.stringify(result))
}

main(process.argv[2])

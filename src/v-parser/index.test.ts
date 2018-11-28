import { test } from '../tester'
import {
  parse,
  select,
  NodeType,
  INodeBold,
  INodeMono,
  INodeStrikethrough,
  INodeCodeBlock,
} from './index'

const text = `
# Header1
block
## Header2

* test
* test

Paragraph and something else. sentence
test *bold\\**
test \`code\\\`\`
test ~strikethrough\\~~ text

\`\`\`js
 code block
\`\`\`

\`\`\`quote:Albert Einstein
Few are those who see with their own eyes and feel with their own hearts.
\`\`\`

One more paragraph. [[http://link.to/123?321][link]]
And image link without description [[image:0d4dbbed6733f4038a8b72dfe1b02030d3bb8fad803e329e3b0bf41f7f8a4452]]

`

test('Markup', (assert) => {
  const result = parse(text)

  assert.equal(select(result, NodeType.Paragraph).length, 6)
  assert.equal(select(result, NodeType.Header).length, 2)
  assert.equal(select(result, NodeType.ListItem).length, 2)

  const bold = select(result, NodeType.Bold) as INodeBold[]
  assert.equal(bold.length, 1)
  assert.equal(bold[0].text, 'bold*')

  const mono = select(result, NodeType.Mono) as INodeMono[]
  assert.equal(mono.length, 1)
  assert.equal(mono[0].text, 'code`')

  const strikethrough = select(result, NodeType.Strikethrough) as INodeStrikethrough[]
  assert.equal(strikethrough.length, 1)
  assert.equal(strikethrough[0].text, 'strikethrough~')

  assert.equal(select(result, NodeType.Link).length, 2)

  const code = select(result, NodeType.CodeBlock) as INodeCodeBlock[]
  assert.equal(code.length, 2)
  assert.equal(code[0].lang, 'js')
  assert.equal(code[1].source, 'Albert Einstein')
})

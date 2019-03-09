import * as React from 'react'
import { Button, examples as buttonExamples } from './Button'
import { Select, examples as selectExamples } from './Select'

export { globalStyles } from './global-styles'

export {
  Button,
  Select,
}

const renderExamples = (title: string, examples: { [description: string]: JSX.Element }) => (
  <div>
    <h1>{title}</h1>
    {Object.entries(examples).map(([description, el], i) => (
      <div key={i}>
        {description && (
          <h2>{description}</h2>
        )}
        {el}
      </div>
    ))}
  </div>
)

export function Library() {
  return (
    <div>
      {renderExamples('Buttons', buttonExamples)}
      {renderExamples('Select', selectExamples)}
    </div>
  )
}

import * as React from 'react'
import { Example } from './Example'
import { CodeBlock } from './CodeBlock'

const codeExample = `
#[macro_use] extern crate tramp;

use tramp::{tramp, Rec};

fn factorial(n: u128) -> u128 {
  fn fac_with_acc(n: u128, acc: u128) -> Rec<u128> {
    if n > 1 {
      rec_call!(fac_with_acc(n - 1, acc * n))
    } else {
      rec_ret!(acc)
    }
  }

  tramp(fac_with_acc(n, 1))
}

assert_eq!(factorial(5), 120);
`.trim()

export function CodeBlockExamples() {
  return (
    <Example section title="CodeBlock">
      <CodeBlock>
        {codeExample}
      </CodeBlock>
    </Example>
  )
}

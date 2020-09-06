import * as React from 'react'
import { Dict } from '@v/utils'
import { Cell } from '@v/reactive'
import { useObservable } from '@v/web-utils'

export const FormContext = React.createContext<Cell<Dict> | undefined>(undefined)

function createForm(values$: Cell<Dict>) {
  interface IProps {
    children: React.ReactNode
  }

  return function Form(props: IProps) {
    return (
      <FormContext.Provider value={values$}>
        {props.children}
      </FormContext.Provider>
    )
  }
}

export function useForm() {
  const [values$] = React.useState(() => new Cell<Dict>({}))

  const [values] = useObservable(() => values$.value$)

  const Form = React.useMemo(() => createForm(values$), [values$])

  return {
    Form,
    values: values || values$.value,
  }
}

export function useFormControl(name: string) {
  const values$ = React.useContext(FormContext)
  if (!values$) {
    throw new Error('FormContext must be provided')
  }

  if (!name) {
    throw new Error('"name" must be provided')
  }

  const [values] = useObservable(() => values$.value$, [values$])

  return {
    value: (values || values$.value)[name] || '',

    setValue(newValue: string) {
      values$.value = {
        ...values$.value,
        [name]: newValue,
      }
    }
  }
}

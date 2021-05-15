import * as React from 'react'
import { Dict } from '@v/utils'
import { Cell } from '@v/reactive'
import { useCell } from '../utils'

type Values = Dict<string | undefined>
type ValuesCell = Cell<Values>

export const FormContext = React.createContext<ValuesCell | undefined>(undefined)

function createForm(values$: ValuesCell) {
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

export function useForm(initialValues: Values = {}) {
  const [values$] = React.useState(() => new Cell<Values>(initialValues))

  const [values] = useCell(values$)

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

  const [values] = useCell(values$)

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

import { createContext } from 'preact';
import { useContext, useState } from 'preact/hooks';
import { JSONObj, JSONValue, formDataToObject } from '../../utils';
import { JSXChildren, JSXRef } from '../../utils/jsx';

type Getter = () => JSONValue;
type Getters = WeakMap<Element, Getter>;

const GettersContext = createContext<Getters | undefined>(undefined);
export function useGettersContext(): Getters {
  const getters = useContext(GettersContext);
  if (!getters) {
    throw new Error('context not initialized');
  }

  return getters;
}

function collectValues(form: HTMLFormElement, getters: Getters): JSONObj {
  const result: JSONObj = {};

  const fd = formDataToObject(new FormData(form));

  for (const control of form.elements) {
    if (control.hasAttribute('disabled')) {
      continue;
    }

    const name = control.getAttribute('name');
    if (!name) {
      console.error('control must have a name', control);
      continue;
    }

    if (control instanceof RadioNodeList) {
      throw new Error(`control "${name}" is RadioNodeList which is unsupported`);
    }

    const getter = getters.get(control);

    result[name] = getter ? getter() : fd[name];
  }

  return result;
}

type FormProps = {
  children: JSXChildren;
  onSubmit: (values: JSONObj) => Promise<void>;
  formRef?: JSXRef<HTMLFormElement>;
};

export function Form({ children, onSubmit, formRef }: FormProps) {
  const [valueExtractors] = useState<Getters>(() => new WeakMap());

  return (
    <GettersContext.Provider value={valueExtractors}>
      <form
        ref={formRef}
        className="form"
        onSubmit={(e) => {
          e.preventDefault();

          // TODO readonly controls while submitting
          void onSubmit(collectValues(e.currentTarget, valueExtractors));
        }}
      >
        {children}
      </form>
    </GettersContext.Provider>
  );
}

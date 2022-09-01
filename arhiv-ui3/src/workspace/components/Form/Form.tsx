import { ComponentChildren, createContext } from 'preact';
import { MutableRef, useContext, useState } from 'preact/hooks';
import { JSONObj, JSONValue } from '../../../scripts/utils';

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

  for (const [name, value] of new FormData(form)) {
    if (typeof value !== 'string') {
      throw new Error('only string values are supported');
    }

    const control = form.elements.namedItem(name);
    if (!control) {
      throw new Error(`control "${name}" is missing`);
    }

    if (control instanceof RadioNodeList) {
      throw new Error(`control "${name}" is RadioNodeList which is unsupported`);
    }

    const getter = getters.get(control);

    result[name] = getter ? getter() : value;
  }

  return result;
}

type FormProps = {
  children: ComponentChildren;
  onSubmit: (values: JSONObj) => Promise<void>;
  formRef?: MutableRef<HTMLFormElement | null>;
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
        {/* Prevent implicit submission of the form */}
        <button type="submit" disabled style="display: none" hidden aria-hidden="true"></button>

        {children}
      </form>
    </GettersContext.Provider>
  );
}

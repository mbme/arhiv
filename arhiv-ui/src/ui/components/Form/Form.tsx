import { createContext } from 'preact';
import { useContext, useState } from 'preact/hooks';
import { JSONObj, JSONValue, formDataToObject, cx } from 'utils';
import { JSXChildren, JSXRef } from 'utils/jsx';

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
      continue;
    }

    if (control instanceof RadioNodeList) {
      throw new Error(`control "${name}" is RadioNodeList which is unsupported`);
    }

    if (control instanceof HTMLElement) {
      const value = control.dataset['value'];

      if (typeof value === 'string') {
        result[name] = JSON.parse(value) as JSONValue;
        continue;
      }
    }

    const getter = getters.get(control);

    result[name] = getter ? getter() : fd[name];
  }

  return result;
}

type FormProps = {
  className?: string;
  children: JSXChildren;
  onSubmit: (values: JSONObj) => Promise<void>;
  formRef?: JSXRef<HTMLFormElement>;
};

export function Form({ className, children, onSubmit, formRef }: FormProps) {
  const [valueExtractors] = useState<Getters>(() => new WeakMap());

  return (
    <GettersContext.Provider value={valueExtractors}>
      <form
        ref={formRef}
        className={cx("form", className)}
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

import { ComponentChildren, createContext } from 'preact';
import { MutableRef, useContext, useEffect, useRef, useState } from 'preact/hooks';
import { JSONObj, JSONValue, Obj } from '../../../scripts/utils';

type ControlValueExtractors = Obj<() => JSONValue>;
const FormContext = createContext<ControlValueExtractors | undefined>(undefined);

export function useFormField<T>(
  name: string,
  extract: (control: T) => JSONValue
): MutableRef<T | null> {
  const controlRef = useRef<T | null>(null);

  const extractorRef = useRef(extract);
  extractorRef.current = extract;

  const formContext = useContext(FormContext);
  if (!formContext) {
    throw new Error('must be used inside the form');
  }

  useEffect(() => {
    formContext[name] = () => {
      if (!controlRef.current) {
        throw new Error(`uninitialized ref to control "${name}"`);
      }

      return extractorRef.current(controlRef.current);
    };

    return () => {
      formContext[name] = undefined;
    };
  }, [formContext, name]);

  return controlRef;
}

function collectValues(valueExtractors: ControlValueExtractors): JSONObj {
  const result: JSONObj = {};

  for (const [name, extractor] of Object.entries(valueExtractors)) {
    if (extractor) {
      result[name] = extractor();
    }
  }

  return result;
}

type FormProps = {
  children: ComponentChildren;
  onSubmit: (values: JSONObj) => Promise<void>;
  formRef?: MutableRef<HTMLFormElement | null>;
};

export function Form({ children, onSubmit, formRef }: FormProps) {
  const [valueExtractors] = useState<ControlValueExtractors>(() => ({}));

  return (
    <FormContext.Provider value={valueExtractors}>
      <form
        ref={formRef}
        className="form"
        onSubmit={(e) => {
          e.preventDefault();

          // TODO block editing while submitting
          void onSubmit(collectValues(valueExtractors));
        }}
      >
        {/* Prevent implicit submission of the form */}
        <button type="submit" disabled style="display: none" hidden aria-hidden="true"></button>

        {children}
      </form>
    </FormContext.Provider>
  );
}

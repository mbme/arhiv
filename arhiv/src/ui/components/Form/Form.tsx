import { useEffect, useRef, useState } from 'react';
import { JSONObj, formDataToObject, cx, JSONValue } from 'utils';
import { JSXChildren, mergeRefs } from 'utils/jsx';
import { HTMLVFormFieldElement } from 'components/Form/FormField';

export const FORM_VIEWPORT_CLASSNAME = 'form-viewport';

function collectValues(form: HTMLFormElement): JSONObj {
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

    if (control instanceof HTMLVFormFieldElement) {
      result[name] = control.value as JSONValue;
      continue;
    }

    if (control instanceof HTMLInputElement && control.type === 'number') {
      result[name] = control.value ? Number.parseInt(control.value, 10) : null;
      continue;
    }

    if (control instanceof HTMLInputElement && control.type === 'checkbox') {
      result[name] = control.checked;
      continue;
    }

    result[name] = fd[name] ?? null;
  }

  return result;
}

class FormDirtyEvent extends CustomEvent<boolean> {
  constructor(public readonly isDirty: boolean) {
    super('formDirty', { detail: isDirty });
  }
}

function isFormDirty(form: HTMLFormElement) {
  return form.dataset['isDirty'] === 'true';
}

export function markFormDirty(form: HTMLFormElement, isDirty: boolean) {
  if (isDirty) {
    form.dataset['isDirty'] = 'true';
  } else {
    delete form.dataset['isDirty'];
  }
  form.dispatchEvent(new FormDirtyEvent(isDirty));
}

export function useIsFormDirty(form: HTMLFormElement | undefined | null) {
  const [isDirty, setIsDirty] = useState(form ? isFormDirty(form) : false);

  useEffect(() => {
    if (!form) {
      setIsDirty(false);
      return;
    }

    setIsDirty(isFormDirty(form));

    const onFormDirty = (e: Event) => {
      setIsDirty((e as FormDirtyEvent).isDirty);
    };

    form.addEventListener('formDirty', onFormDirty);

    return () => {
      form.removeEventListener('formDirty', onFormDirty);
    };
  }, [form]);

  return isDirty;
}

type FormProps = {
  className?: string;
  children: JSXChildren;
  onSubmit: (values: JSONObj) => Promise<void> | void;
  formRef?: React.Ref<HTMLFormElement>;
};

export function Form({ className, children, onSubmit, formRef }: FormProps) {
  const ref = useRef<HTMLFormElement | null>(null);

  useEffect(() => {
    const form = ref.current;
    if (!form) {
      throw new Error('form is missing');
    }

    const onInput = () => {
      if (!isFormDirty(form)) {
        markFormDirty(form, true);
      }
    };

    // NOTE: regular onInput doesn't catch custom input events dispatched by the v-form-field
    form.addEventListener('input', onInput);

    return () => {
      form.removeEventListener('input', onInput);
    };
  }, []);

  return (
    <form
      ref={mergeRefs(ref, formRef)}
      className={cx('form', className)}
      onSubmit={(e) => {
        e.preventDefault();
        e.stopPropagation();

        const form = e.currentTarget;

        // TODO readonly controls while submitting
        void Promise.resolve(onSubmit(collectValues(e.currentTarget))).then(() => {
          markFormDirty(form, false);
        });
      }}
      onReset={(e) => {
        markFormDirty(e.currentTarget, false);
      }}
    >
      {children}
    </form>
  );
}

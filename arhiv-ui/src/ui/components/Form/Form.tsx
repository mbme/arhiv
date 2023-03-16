import { useEffect, useState } from 'react';
import { JSONObj, formDataToObject, cx } from 'utils';
import { JSXChildren, JSXRef } from 'utils/jsx';
import { HTMLVFormFieldElement } from 'components/Form/FormField';

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
      result[name] = control.value;
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

function markFormDirty(form: HTMLFormElement, isDirty: boolean) {
  form.dataset['isDirty'] = isDirty ? 'true' : undefined;
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
  formRef?: JSXRef<HTMLFormElement>;
};

export function Form({ className, children, onSubmit, formRef }: FormProps) {
  return (
    <form
      ref={formRef}
      className={cx('form', className)}
      onSubmit={(e) => {
        e.preventDefault();

        const form = e.currentTarget;

        // TODO readonly controls while submitting
        void Promise.resolve(onSubmit(collectValues(e.currentTarget))).then(() => {
          markFormDirty(form, false);
        });
      }}
      onInput={(e) => {
        if (!isFormDirty(e.currentTarget)) {
          markFormDirty(e.currentTarget, true);
        }
      }}
    >
      {children}
    </form>
  );
}

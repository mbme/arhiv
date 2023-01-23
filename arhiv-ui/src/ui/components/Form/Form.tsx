import { JSONObj, formDataToObject, cx } from 'utils';
import { JSXChildren, JSXRef } from 'utils/jsx';
import { HTMLVRefInputElement } from 'components/Form/v-ref-input/index';

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

    if (control instanceof HTMLVRefInputElement) {
      const value = control.refs;

      if (control.multiple) {
        result[name] = value;
      } else {
        result[name] = value[0] ?? null;
      }
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

    result[name] = fd[name];
  }

  return result;
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

        // TODO readonly controls while submitting
        void onSubmit(collectValues(e.currentTarget));
      }}
    >
      {children}
    </form>
  );
}

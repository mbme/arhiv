/* eslint-disable @typescript-eslint/no-floating-promises */

import 'utils/test-env-setup';

import { afterEach, describe, it } from 'node:test';
import assert from 'node:assert';
import { render, cleanup, waitFor } from '@testing-library/react';
import { formDataToObject } from 'utils';
import { FormField, HTMLVFormFieldElement } from './FormField';

afterEach(cleanup);

function findBySelector<T extends Element>(container: Element, selector: string): T {
  const el = container.querySelector<T>(selector);

  if (!el) {
    throw new Error(`can't find anything matching selector '${selector}'`);
  }

  return el;
}

describe('FormField', () => {
  it('stores value in a form', () => {
    const { container } = render(
      <form>
        <FormField name="test" defaultValue="initial" />
      </form>,
    );

    const form = findBySelector<HTMLFormElement>(container, 'form');
    assert(!!form.elements.namedItem('test'));

    {
      const result = formDataToObject(new FormData(form));
      assert.equal(JSON.parse(result['test']!), 'initial');
    }

    const field = findBySelector<HTMLVFormFieldElement<number>>(container, 'v-form-field');
    field.value = 123;

    {
      const result = formDataToObject(new FormData(form));
      assert.equal(JSON.parse(result['test']!), 123);
    }
  });

  it("doesn't store value in a form if disabled", () => {
    const { container } = render(
      <form>
        <FormField name="test" defaultValue="initial" disabled />
      </form>,
    );

    const form = findBySelector<HTMLFormElement>(container, 'form');

    {
      const result = formDataToObject(new FormData(form));
      assert.equal(result['test'], undefined);
    }

    // value assignment also must not update the form value
    const field = findBySelector<HTMLVFormFieldElement<number>>(container, 'v-form-field');
    field.value = 123;

    {
      const result = formDataToObject(new FormData(form));
      assert.equal(result['test'], undefined);
    }
  });

  it('returns to default value on form reset', async () => {
    const { container } = render(
      <form>
        <FormField name="test" defaultValue="initial" />
      </form>,
    );

    const form = findBySelector<HTMLFormElement>(container, 'form');

    const field = findBySelector<HTMLVFormFieldElement<string>>(container, 'v-form-field');
    field.value = '123';

    await waitFor(() => field.value === '123');

    {
      const result = formDataToObject(new FormData(form));
      assert.equal(JSON.parse(result['test']!), '123');
    }

    form.reset();

    {
      const result = formDataToObject(new FormData(form));
      assert.equal(JSON.parse(result['test']!), 'initial');
    }
  });

  it('invalidates form if required & empty', async () => {
    const { container } = render(
      <form>
        <FormField name="test" />
      </form>,
    );

    const form = findBySelector<HTMLFormElement>(container, 'form');

    {
      const result = formDataToObject(new FormData(form));
      assert.equal(JSON.parse(result['test']!), null);
    }

    const field = findBySelector<HTMLVFormFieldElement<string>>(container, 'v-form-field');
    field.setAttribute('required', '');

    await waitFor(() => !form.checkValidity());

    assert(!form.checkValidity());

    field.value = '123';

    assert(form.checkValidity());
  });
});

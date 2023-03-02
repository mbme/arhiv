import 'global-jsdom/register';
global.Event = window.Event;

import test from 'ava';
import { render, cleanup, waitFor } from '@testing-library/preact';
import { formDataToObject } from 'utils';
import './v-form-field';
import { HTMLVFormFieldElement } from './v-form-field';

test.after(cleanup);

function findBySelector<T extends Element>(container: Element, selector: string): T {
  const el = container.querySelector<T>(selector);

  if (!el) {
    throw new Error(`can't find anything matching selector '${selector}'`);
  }

  return el;
}

test('stores value in a form', (t) => {
  const { container } = render(
    <form>
      <v-form-field name="test" defaultValue='"initial"' />
    </form>
  );

  const form = findBySelector<HTMLFormElement>(container, 'form');
  t.truthy(form.elements.namedItem('test'));

  {
    const result = formDataToObject(new FormData(form));
    t.is(JSON.parse(result['test']!), 'initial');
  }

  const field = findBySelector<HTMLVFormFieldElement>(container, 'v-form-field');
  field.value = 123;

  {
    const result = formDataToObject(new FormData(form));
    t.is(JSON.parse(result['test']!), 123);
  }
});

test('dispatches change event', async (t) => {
  let changeCounter = 0;

  const { container } = render(
    <form>
      <v-form-field
        name="test"
        onChange={() => {
          changeCounter += 1;
        }}
      />
    </form>
  );

  const field = findBySelector<HTMLVFormFieldElement>(container, 'v-form-field');
  field.value = 123;
  await waitFor(() => field.value === 123);
  t.is(changeCounter, 1);

  const form = findBySelector<HTMLFormElement>(container, 'form');
  form.reset();
  t.is(changeCounter, 2);
});

test("doesn't store value in a form if disabled", (t) => {
  const { container } = render(
    <form>
      <v-form-field name="test" defaultValue='"initial"' disabled />
    </form>
  );

  const form = findBySelector<HTMLFormElement>(container, 'form');

  {
    const result = formDataToObject(new FormData(form));
    t.is(result['test'], undefined);
  }

  // value assignment also must not update the form value
  const field = findBySelector<HTMLVFormFieldElement>(container, 'v-form-field');
  field.value = 123;

  {
    const result = formDataToObject(new FormData(form));
    t.is(result['test'], undefined);
  }
});

test('returns to default value on form reset', async (t) => {
  const { container } = render(
    <form>
      <v-form-field name="test" defaultValue='"initial"' />
    </form>
  );

  const form = findBySelector<HTMLFormElement>(container, 'form');

  const field = findBySelector<HTMLVFormFieldElement>(container, 'v-form-field');
  field.value = '123';

  await waitFor(() => field.value === '123');

  {
    const result = formDataToObject(new FormData(form));
    t.is(JSON.parse(result['test']!), '123');
  }

  form.reset();

  {
    const result = formDataToObject(new FormData(form));
    t.is(JSON.parse(result['test']!), 'initial');
  }
});

test('invalidates form if required & empty', async (t) => {
  const { container } = render(
    <form>
      <v-form-field name="test" />
    </form>
  );

  const form = findBySelector<HTMLFormElement>(container, 'form');

  {
    const result = formDataToObject(new FormData(form));
    t.is(JSON.parse(result['test']!), null);
  }

  const field = findBySelector<HTMLVFormFieldElement>(container, 'v-form-field');
  field.setAttribute('required', '');

  await waitFor(() => !form.checkValidity());

  t.false(form.checkValidity());

  field.value = '123';

  t.true(form.checkValidity());
});

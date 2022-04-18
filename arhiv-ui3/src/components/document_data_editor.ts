import { setQueryParam } from '../scripts/utils';

export function initDataEditor(form: HTMLFormElement) {
  const subtypeSelect: HTMLSelectElement | null = form.querySelector('select#subtype');

  if (!subtypeSelect) {
    throw new Error('subtype select must be present');
  }

  if (!form.dataset.subtypes) {
    throw new Error('form must have attribute data-subtypes');
  }

  const subtypes = JSON.parse(form.dataset.subtypes) as string[];

  const updateFieldsVisibility = () => {
    if (!subtypes.length) {
      return;
    }

    const subtype = subtypeSelect.value;

    for (const fieldLabel of form.querySelectorAll<HTMLLabelElement>('label[data-for-subtypes]')) {
      if (!fieldLabel.dataset.forSubtypes) {
        throw new Error('form field must have attribute data-for-subtypes');
      }

      const forSubtypes = JSON.parse(fieldLabel.dataset.forSubtypes) as string[];

      const shouldBeVisible = forSubtypes.includes(subtype);

      fieldLabel.toggleAttribute('hidden', !shouldBeVisible);
    }
  };

  subtypeSelect.addEventListener('change', () => {
    form.action = setQueryParam(form.action, 'subtype', subtypeSelect.value);

    updateFieldsVisibility();
  });

  updateFieldsVisibility();

  // autofocus the first visible field
  const firstVisibleLabel = form.querySelector<HTMLLabelElement>(
    'label[data-for-subtypes]:not([hidden])'
  );
  firstVisibleLabel?.click();
}

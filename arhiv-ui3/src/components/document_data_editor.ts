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

    for (const field of form.querySelectorAll<HTMLElement>('[data-for-subtypes]')) {
      if (!field.dataset.forSubtypes) {
        throw new Error('form field must have attribute data-for-subtypes');
      }

      const forSubtypes = JSON.parse(field.dataset.forSubtypes) as string[];

      const shouldBeVisible = forSubtypes.includes(subtype);

      field.toggleAttribute('hidden', !shouldBeVisible);
    }
  };

  updateFieldsVisibility();

  subtypeSelect.addEventListener('change', () => {
    form.action = setQueryParam(form.action, 'subtype', subtypeSelect.value);

    updateFieldsVisibility();
  });
}

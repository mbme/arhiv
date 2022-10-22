import { useState } from 'preact/hooks';
import { noop, formDataToObject } from '../../scripts/utils';
import { useSessionState } from '../hooks';
import { JSXChildren } from '../jsx';
import { CardContext } from '../workspace-reducer';
import { Button, IconButton } from './Button';
import { DateTime } from './DateTime';
import { Dialog } from './Dialog';
import { Checkbox } from './Form/Checkbox';
import { Select } from './Form/Select';
import { Icon, ICON_VARIANTS } from './Icon';
import { Link } from './Link';
import { QueryError } from './QueryError';
import { Ref } from './Ref';

export function ComponentsDemo() {
  return (
    <div className="bg-white h-full overflow-auto">
      <div className="components-demo">
        <div>
          <h1>Form controls</h1>
          <FormControlsDemo />
        </div>
        <div>
          <h1>Button</h1>
          <div className="examples">
            <Button variant="primary">Primary</Button>
            <Button variant="simple">Simple</Button>
            <Button variant="text">Text</Button>
          </div>

          <h1>Button: with leading icon</h1>
          <div className="examples">
            <Button variant="primary" leadingIcon="web">
              Primary
            </Button>
            <Button variant="simple" leadingIcon="web">
              Simple
            </Button>
            <Button variant="text" leadingIcon="web">
              Text
            </Button>
          </div>

          <h1>Button: with trailing icon</h1>
          <div className="examples">
            <Button variant="primary" trailingIcon="web">
              Primary
            </Button>
            <Button variant="simple" trailingIcon="web">
              Simple
            </Button>
            <Button variant="text" trailingIcon="web">
              Text
            </Button>
          </div>

          <h1>Button: disabled</h1>
          <div className="examples">
            <Button variant="primary" disabled>
              Primary
            </Button>
            <Button variant="simple" disabled>
              Simple
            </Button>
            <Button variant="text" disabled>
              Text
            </Button>
          </div>

          <h1>Button: busy</h1>
          <div className="examples">
            <Button variant="primary" busy>
              Primary
            </Button>
            <Button variant="simple" busy>
              Simple
            </Button>
            <Button variant="text" busy>
              Text
            </Button>
          </div>

          <h1>Button: alarming</h1>
          <div className="examples">
            <Button variant="primary" alarming>
              Primary
            </Button>
            <Button variant="simple" alarming>
              Simple
            </Button>
            <Button variant="text" alarming>
              Text
            </Button>
          </div>

          <h1>Icon Button</h1>
          <div className="examples">
            <IconButton icon="web" className="text-red-700" />

            <IconButton icon="web" size="lg" />
          </div>
        </div>

        <div>
          <h1>QueryError</h1>
          <div className="examples">
            <QueryError error="Something is wrong :(" />
          </div>
        </div>

        <div>
          <h1>DateTime</h1>
          <div className="examples">
            <DateTime datetime={new Date().toISOString()} />
          </div>

          <h1>DateTime: relative</h1>
          <div className="examples">
            <DateTime datetime={new Date(Date.now() - 9999000).toISOString()} relative />
          </div>
        </div>

        <div>
          <h1>{ICON_VARIANTS.length} Icons</h1>
          <div className="examples-grid mt-8">
            {ICON_VARIANTS.map((variant) => (
              <div key={variant} className="flex flex-col gap-4 items-center text-center">
                <Icon variant={variant} className="w-12 h-12 block" />
                <div className="font-mono text-xs">{variant}</div>
              </div>
            ))}
          </div>
        </div>

        <div>
          <h1>Dialog</h1>
          <div className="examples">
            <DialogExample buttonText="Dialog with buttons">
              <div className="modal-content">Hello world!</div>
              <div className="modal-buttons">
                <Button variant="simple">Cancel</Button>
                <Button variant="primary">Test</Button>
              </div>
            </DialogExample>

            <DialogExample buttonText="Alarming dialog" alarming>
              <div className="modal-content">Hello world!</div>
              <div className="modal-buttons">
                <Button variant="simple">Cancel</Button>
                <Button variant="primary" alarming>
                  Test
                </Button>
              </div>
            </DialogExample>
          </div>
        </div>

        <div>
          <CardContextMock>
            <h1>Ref</h1>

            <div className="examples">
              <Ref id="12342" documentType="note" subtype="" documentTitle="Very important note" />
            </div>

            <h1>Ref with subtype</h1>

            <div className="examples">
              <Ref
                id="12342"
                documentType="note"
                subtype="other"
                documentTitle="Very important note"
              />
            </div>

            <h1>Ref to erased document</h1>

            <div className="examples">
              <Ref id="12342321" documentType="" subtype="" documentTitle="12342321" />
            </div>

            <h1>Ref with subtype</h1>

            <div className="examples">
              <Ref
                id="12342"
                documentType="note"
                documentTitle=""
                subtype="other"
                title="custom title"
                description="Note with custom description"
              />
            </div>

            <h1>External link</h1>

            <div className="examples">
              <Link url="https://example.com" title="some title" description="Goto link" />
            </div>
          </CardContextMock>
        </div>
      </div>
    </div>
  );
}

const CardContextMock = ({ children }: { children: JSXChildren }) => {
  return (
    <CardContext.Provider
      value={{ card: { variant: 'document', id: 1, documentId: 'test' }, dispatch: noop }}
    >
      {children}
    </CardContext.Provider>
  );
};

type DialogExampleProps = {
  buttonText: string;
  children: JSXChildren;
  alarming?: boolean;
};
function DialogExample({ buttonText, children, alarming }: DialogExampleProps) {
  const [showModal, setShowModal] = useState(false);

  return (
    <>
      <Button variant="primary" onClick={() => setShowModal(true)} alarming={alarming}>
        {buttonText}
      </Button>

      {showModal && (
        <Dialog title="Dialog example" onHide={() => setShowModal(false)} alarming={alarming}>
          {children}
        </Dialog>
      )}
    </>
  );
}

function FormControlsDemo() {
  const [disabled, setDisabled] = useSessionState<boolean>('demo-form-disabled', false);
  const [readonly, setReadonly] = useSessionState<boolean>('demo-form-readonly', false);
  const [required, setRequired] = useSessionState<boolean>('demo-form-required', false);

  const [data, setData] = useState('');

  return (
    <form
      className="form flex flex-col gap-8"
      onSubmit={(e) => {
        e.preventDefault();

        const fd = formDataToObject(new FormData(e.currentTarget));

        setData(JSON.stringify(fd, null, 2));
      }}
    >
      <div className="flex gap-4 bg-rose-50 px-2 py-4">
        <label className="flex gap-1 items-center">
          <input type="checkbox" checked={disabled} onChange={() => setDisabled(!disabled)} />
          Disabled
        </label>

        <label className="flex gap-1 items-center">
          <input type="checkbox" checked={readonly} onChange={() => setReadonly(!readonly)} />
          Readonly
        </label>

        <label className="flex gap-1 items-center">
          <input type="checkbox" checked={required} onChange={() => setRequired(!required)} />
          Required
        </label>
      </div>

      <label>
        Editor
        <v-editor
          className="field"
          name="editor"
          required={required}
          disabled={disabled}
          readonly={readonly}
          placeholder="Type something"
        />
      </label>

      <label className="flex items-center gap-2">
        Text input
        <input
          className="field flex-1"
          name="text"
          type="text"
          placeholder="Type something"
          required={required}
          disabled={disabled}
          readOnly={readonly}
        />
      </label>

      <label className="flex items-center gap-2">
        Number input
        <input
          className="field"
          name="number"
          type="number"
          placeholder="numbers"
          min={0}
          max={100}
          step={1}
          required={required}
          disabled={disabled}
          readOnly={readonly}
        />
      </label>

      <label className="flex items-center gap-2">
        Select
        <Select
          className="field"
          name="select"
          required={required}
          disabled={disabled}
          readonly={readonly}
          options={['1', '2']}
        />
      </label>

      <label className="flex items-center gap-2">
        <Checkbox
          className="field"
          name="checkbox"
          required={required}
          disabled={disabled}
          readonly={readonly}
        />
        Checkbox
      </label>

      <div className="buttons">
        <Button variant="simple" type="reset">
          Reset
        </Button>

        <Button type="submit" variant="primary">
          SUBMIT
        </Button>
      </div>

      <pre hidden={!data}>
        <code>{data}</code>
      </pre>
    </form>
  );
}

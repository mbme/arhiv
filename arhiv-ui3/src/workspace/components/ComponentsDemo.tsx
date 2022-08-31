import { ComponentChildren } from 'preact';
import { useState } from 'preact/hooks';
import { noop } from '../../scripts/utils';
import '../../scripts/v-editor';
import { CardContext } from '../workspace-reducer';
import { Button } from './Button';
import { DateTime } from './DateTime';
import { Dialog } from './Dialog';
import { Icon, ICON_VARIANTS } from './Icon';
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
              <Ref id="12342" documentType="note" subtype="" title="Very important note" />
            </div>

            <h1>Ref with subtype</h1>

            <div className="examples">
              <Ref id="12342" documentType="note" subtype="other" title="Very important note" />
            </div>
          </CardContextMock>
        </div>
      </div>
    </div>
  );
}

const CardContextMock = ({ children }: { children: ComponentChildren }) => {
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
  children: ComponentChildren;
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
  const [data, setData] = useState('');

  return (
    <form
      className="flex flex-col gap-8"
      onSubmit={(e) => {
        e.preventDefault();
        const fd = new FormData(e.target as HTMLFormElement);

        setData(JSON.stringify(Object.fromEntries(fd), null, 2));
      }}
    >
      <label>
        Editor
        <v-editor name="editor" value="" required />
      </label>

      <label className="flex items-center gap-2">
        Text input
        <input name="text" type="text" placeholder="Some initial text" />
      </label>

      <label className="flex items-center gap-2">
        Number input
        <input name="number" type="number" placeholder="numbers" min={0} max={100} step={1} />
      </label>

      <label className="flex items-center gap-2">
        Select
        <select name="select">
          <option value="">Empty value</option>
          <option value="1">Value 1</option>
          <option value="2">Value 2</option>
        </select>
      </label>

      <label className="flex items-center gap-2">
        <input name="checkbox" type="checkbox" />
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

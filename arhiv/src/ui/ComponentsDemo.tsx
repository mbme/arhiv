import { Suspense, useState } from 'react';
import { DocumentId, DocumentType } from 'dto';
import { setQueryParam, JSONObj } from 'utils';
import { useScrollRestoration, useSessionState } from 'utils/hooks';
import { JSXChildren } from 'utils/jsx';
import { Button, IconButton } from 'components/Button';
import { DateTime } from 'components/DateTime';
import { Dialog } from 'components/Dialog';
import { Badge } from 'components/Badge';
import { Form } from 'components/Form/Form';
import { Checkbox } from 'components/Form/Checkbox';
import { Select } from 'components/Form/Select';
import { Editor } from 'components/Form/Editor';
import { RefInput } from 'components/Form/RefInput';
import { Icon, ICON_VARIANTS } from 'components/Icon';
import { Link } from 'components/Link';
import { QueryError } from 'components/QueryError';
import { Ref } from 'components/Ref';
import { Spoiler } from 'components/Spoiler';
import { Toaster, showToast } from 'components/Toaster';
import { SuspenseCacheProvider } from 'components/SuspenseCacheProvider';

export function ComponentsDemo() {
  const [wrapperEl, setWrapperEl] = useState<HTMLElement | null>(null);

  useScrollRestoration(wrapperEl, 'components-demo-scroll');

  return (
    <SuspenseCacheProvider cacheId="components-demo">
      <Suspense fallback={<div />}>
        <div className="var-bg-color h-full overflow-auto" ref={setWrapperEl}>
          <IconButton
            icon="x"
            className="fixed top-2 right-4"
            onClick={() => {
              setQueryParam('DEMO', undefined);
              window.location.reload();
            }}
          />
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

                <IconButton icon="web" size="lg" title="lg" />

                <IconButton icon="web" size="xl" title="xl" />
              </div>
            </div>

            <div>
              <h1>Badges</h1>
              <div className="examples">
                <Badge label="unchecked" />
                <Badge label="checked" checked />
                <Badge label="unchecked sm" size="sm" />
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
                <DialogExample
                  buttonText="Dialog with buttons"
                  buttons={
                    <>
                      <Button variant="simple">Cancel</Button>
                      <Button variant="primary">Test</Button>
                    </>
                  }
                >
                  Hello world!
                </DialogExample>

                <DialogExample
                  buttonText="Alarming dialog"
                  alarming
                  buttons={
                    <>
                      <Button variant="simple">Cancel</Button>
                      <Button variant="primary" alarming>
                        Test
                      </Button>
                    </>
                  }
                >
                  Hello world!
                </DialogExample>
              </div>
            </div>

            <div>
              <h1>Spoiler</h1>
              <div className="examples">
                <Spoiler heading={<div>Spoiler 1</div>}>
                  <h1>CONTENT</h1>
                </Spoiler>
              </div>
            </div>

            <div>
              <h1>Toaster</h1>
              <div className="examples">
                <Toaster />

                <Button
                  variant="primary"
                  onClick={() => {
                    showToast({ level: 'info', message: 'Info message ' });
                  }}
                >
                  Add info toast
                </Button>

                <Button
                  variant="primary"
                  alarming
                  onClick={() => {
                    showToast({ level: 'warn', message: 'Warn message ' });
                  }}
                >
                  Add warn toast
                </Button>
              </div>
            </div>

            <div>
              <h1>Ref</h1>

              <div className="examples">
                <Ref
                  documentId={'test123' as DocumentId}
                  documentType={'note' as DocumentType}
                  documentTitle="Very important note"
                />
              </div>

              <h1>Ref to erased document</h1>

              <div className="examples">
                <Ref
                  documentId={'test123' as DocumentId}
                  documentType={'' as DocumentType}
                  documentTitle="12342321"
                />
              </div>

              <h1>Ref with custom description</h1>

              <div className="examples">
                <Ref
                  documentId={'test123' as DocumentId}
                  documentType={'note' as DocumentType}
                  documentTitle=""
                  description="Note with custom description"
                />
              </div>

              <h1>Long Ref line wrap</h1>

              <div className="examples">
                <div className="w-8/12 overflow-auto block border border-indigo-500">
                  Some looooooooooong text and{' '}
                  <Ref
                    documentId={'test123' as DocumentId}
                    documentType={'attachment' as DocumentType}
                    documentTitle="298099334_5292996204070913_386679234432423424242432234323333333333333333_32423-4061939409_n.jpg"
                  />{' '}
                  and lorem ipsum
                </div>
              </div>

              <h1>External link</h1>

              <div className="examples">
                <Link url="https://example.com" title="some title">
                  Goto link
                </Link>
              </div>
            </div>
          </div>
        </div>
      </Suspense>
    </SuspenseCacheProvider>
  );
}

type DialogExampleProps = {
  buttonText: string;
  children: JSXChildren;
  alarming?: boolean;
  buttons?: JSXChildren;
};
function DialogExample({ buttonText, children, alarming, buttons }: DialogExampleProps) {
  const [showModal, setShowModal] = useState(false);

  return (
    <>
      <Button
        variant="primary"
        onClick={() => {
          setShowModal(true);
        }}
        alarming={alarming}
      >
        {buttonText}
      </Button>

      {showModal && (
        <Dialog
          title="Dialog example"
          onHide={() => {
            setShowModal(false);
          }}
          alarming={alarming}
          buttons={buttons}
        >
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
    <Form
      className="flex flex-col gap-8"
      onSubmit={(values: JSONObj) => {
        setData(JSON.stringify(values, null, 2));
      }}
    >
      <div className="flex gap-4 bg-rose-50 dark:bg-slate-900 px-2 py-4">
        <label className="flex gap-1 items-center">
          <input
            type="checkbox"
            checked={disabled}
            onChange={() => {
              setDisabled(!disabled);
            }}
          />
          Disabled
        </label>

        <label className="flex gap-1 items-center">
          <input
            type="checkbox"
            checked={readonly}
            onChange={() => {
              setReadonly(!readonly);
            }}
          />
          Readonly
        </label>

        <label className="flex gap-1 items-center">
          <input
            type="checkbox"
            checked={required}
            onChange={() => {
              setRequired(!required);
            }}
          />
          Required
        </label>
      </div>

      <label>
        Editor
        <Editor
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
          className="flex-1"
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
          name="select"
          required={required}
          disabled={disabled}
          readonly={readonly}
          options={['1', '2']}
        />
      </label>

      <label className="flex items-center gap-2">
        <Checkbox name="checkbox" required={required} disabled={disabled} readonly={readonly} />
        Checkbox
      </label>

      <label className="flex items-center gap-2">
        Ref picker input
        <RefInput
          documentTypes={['note', 'book'] as DocumentType[]}
          name="note-ref"
          required={required}
          disabled={disabled}
          readonly={readonly}
        />
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
    </Form>
  );
}

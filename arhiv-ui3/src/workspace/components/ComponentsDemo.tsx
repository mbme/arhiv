import { Button } from './Button';
import { DateTime } from './DateTime';
import { Icon, ICON_VARIANTS } from './Icon';
import { QueryError } from './QueryError';

export function ComponentsDemo() {
  return (
    <div className="bg-white h-full overflow-auto">
      <div className="components-demo">
        <div>
          <h1>Button</h1>
          <div className="examples">
            <Button variant="prime">Prime</Button>
            <Button variant="simple">Simple</Button>
            <Button variant="text">Text</Button>
          </div>

          <h1>Button: with icons</h1>
          <div className="examples">
            <Button variant="prime" icon="web">
              Prime
            </Button>
            <Button variant="simple" icon="web">
              Simple
            </Button>
            <Button variant="text" icon="web">
              Text
            </Button>
          </div>

          <h1>Button: disabled</h1>
          <div className="examples">
            <Button variant="prime" disabled>
              Prime
            </Button>
            <Button variant="simple" disabled>
              Simple
            </Button>
            <Button variant="text" disabled>
              Text
            </Button>
          </div>

          <h1>Button: loading</h1>
          <div className="examples">
            <Button variant="prime" loading>
              Prime
            </Button>
            <Button variant="simple" loading>
              Simple
            </Button>
            <Button variant="text" loading>
              Text
            </Button>
          </div>

          <h1>Button: alarming</h1>
          <div className="examples">
            <Button variant="prime" alarming>
              Prime
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
      </div>
    </div>
  );
}

import { JSXChildren } from 'utils/jsx';

interface Props {
  heading: string;
  children: JSXChildren;
}
export function LoginContainer({ heading, children }: Props) {
  return (
    <div className="h-full pt-12 sm:pt-32 pb-16 px-8 overflow-y-auto">
      <div className="flex flex-col items-center justify-center w-full max-w-md mx-auto">
        <img
          src={`${window.CONFIG.basePath}/favicon.svg`}
          alt="Arhiv logo"
          className="block size-24 rounded-md shadow-lg mb-8"
        />

        <h1 className="heading-1 mb-4">
          {heading} {window.CONFIG.devMode && 'DEV MODE'}
        </h1>

        <div className="section-heading mb-8">{window.CONFIG.storageDir}</div>

        {children}
      </div>
    </div>
  );
}

import { JSXChildren } from 'utils/jsx';

interface Props {
  heading: string;
  children: JSXChildren;
}
export function LoginContainer({ heading, children }: Props) {
  return (
    <div className="flex flex-col items-center justify-center pt-32">
      <img
        src={`${window.CONFIG.basePath}/favicon.svg`}
        alt="Arhiv logo"
        className="size-24 rounded-md shadow-lg mb-8"
      />

      <h1 className="heading-1 mb-4">{heading}</h1>

      <div className="section-heading mb-8">{window.CONFIG.storageDir}</div>

      {children}
    </div>
  );
}

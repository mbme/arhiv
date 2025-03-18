import { JSXChildren } from 'utils/jsx';

interface Props {
  children: JSXChildren;
}
export function LoginContainer({ children }: Props) {
  return (
    <div className="flex flex-col items-center justify-center pt-32">
      <img
        src={`${window.CONFIG.basePath}/favicon.svg`}
        alt="Arhiv logo"
        className="size-24 rounded-md shadow-lg mb-8"
      />

      {children}
    </div>
  );
}

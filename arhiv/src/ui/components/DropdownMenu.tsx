import { Fragment } from 'react';
import { Menu, Transition } from '@headlessui/react';
import { Callback, cx } from 'utils';
import { Icon, IconVariant } from './Icon';
import { IconButton } from './Button';

type MenuItem = {
  text: string;
  icon?: IconVariant;
  alarming?: boolean;
  onClick: Callback;
};

export type DropdownOptions = ReadonlyArray<MenuItem | false>;

type DropdownMenuProps = {
  icon?: IconVariant;
  options: DropdownOptions;
  align: 'bottom-left' | 'bottom-right';
};

export function DropdownMenu({ icon = 'more', align, options }: DropdownMenuProps) {
  return (
    <Menu as="div" className="relative z-10">
      {({ open }) => (
        <>
          <Menu.Button
            as={IconButton}
            icon={icon}
            size="lg"
            className={cx({ 'var-item-active-bg-color': open })}
          />

          <Transition
            as={Fragment}
            enter="transition ease-out duration-100"
            enterFrom="transform opacity-0 scale-95"
            enterTo="transform opacity-100 scale-100"
            leave="transition ease-in duration-75"
            leaveFrom="transform opacity-100 scale-100"
            leaveTo="transform opacity-0 scale-95"
          >
            <Menu.Items
              className={cx(
                'rounded w-max flex flex-col gap-2 drop-shadow py-2 absolute mt-2 bg-white shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none',
                {
                  'left-0 origin-top-left': align === 'bottom-left',
                  'right-0 origin-top-right': align === 'bottom-right',
                },
              )}
            >
              {options.map((option, index) => {
                if (!option) {
                  return null;
                }

                return (
                  <Menu.Item key={index}>
                    {({ active }) => (
                      <button
                        type="button"
                        className={cx(
                          'flex items-center gap-5 cursor-pointer px-4 py-2 whitespace-nowrap',
                          {
                            'text-blue-700 hover:bg-sky-100': !option.alarming,
                            'text-red-700 hover:bg-red-300': option.alarming,
                            'var-item-active-bg-color': active,
                          },
                        )}
                        onClick={() => option.onClick()}
                      >
                        {option.icon ? <Icon variant={option.icon} /> : <div className="w-5" />}

                        {option.text}
                      </button>
                    )}
                  </Menu.Item>
                );
              })}
            </Menu.Items>
          </Transition>
        </>
      )}
    </Menu>
  );
}

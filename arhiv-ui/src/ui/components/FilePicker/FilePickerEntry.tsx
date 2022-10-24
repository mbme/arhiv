import { Callback, cx, formatBytes } from '../../utils';
import { DirEntry } from '../../../dto';
import { Icon } from '../Icon';

type Props = {
  entry: DirEntry;
  onClick: Callback;
};
export function FilePickerEntry({ entry, onClick }: Props) {
  return (
    <div
      onClick={entry.isReadable ? onClick : undefined}
      className={cx(
        'break-all cursor-pointer p-3 flex items-center text-sm font-mono odd:bg-gray-50',
        {
          'focus:text-blue-700 hover:text-blue-700': entry.isReadable,
          'text-red-700 cursor-default ': !entry.isReadable,
        }
      )}
    >
      {entry.typeName === 'Dir' ? (
        <Icon variant="folder" className="flex-shrink-0 mr-2 mb-1" />
      ) : (
        <div className="w-5 flex-shrink-0">&nbsp;</div>
      )}

      <div className="flex flex-wrap justify-between w-full">
        {entry.name}

        {entry.typeName === 'Symlink' && (
          <span className="text-indigo-700">&rarr; {entry.linksTo}</span>
        )}

        {'size' in entry && typeof entry.size === 'number' && (
          <div className="text-gray-500">{formatBytes(entry.size)}</div>
        )}
      </div>
    </div>
  );
}

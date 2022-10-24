import { DirEntry } from '../../../dto';

type Props = {
  dir?: string;
  entries?: readonly DirEntry[];
};
export function FilePickerHead({ dir, entries }: Props) {
  const counts = entries?.reduce(
    (acc, entry) => {
      switch (entry.typeName) {
        case 'File':
          acc.files += 1;
          break;

        case 'Dir':
          acc.dirs += 1;
          break;

        case 'Symlink':
          acc.links += 1;
          break;
      }

      return acc;
    },
    { files: 0, dirs: 0, links: 0 }
  );

  return (
    <div>
      <div className="font-mono text-lg font-bold">{dir}</div>
      {counts && (
        <div className="flex gap-4 text-sm font-mono">
          <span>{counts.dirs} dirs</span>
          <span>{counts.files} files</span>
          <span>{counts.links} links</span>
        </div>
      )}
    </div>
  );
}

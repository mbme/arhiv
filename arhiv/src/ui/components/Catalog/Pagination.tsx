import { Button } from '../Button';
import { Icon } from '../Icon';

type PaginationProps = {
  page: number;
  hasMore: boolean;
  onClick: (nextPage: number) => void;
};
export function Pagination({ page, hasMore, onClick }: PaginationProps) {
  return (
    <div className="flex items-center justify-center gap-6 font-mono py-3 var-bg-color empty:hidden">
      {page > 0 && (
        <>
          <Button
            variant="text"
            onClick={() => {
              onClick(page - 1);
            }}
          >
            <Icon variant="arrow-left" className="mr-1" />
            prev
          </Button>

          <div>Page {page}</div>
        </>
      )}

      {hasMore && (
        <Button
          variant="text"
          onClick={() => {
            onClick(page + 1);
          }}
        >
          next
          <Icon variant="arrow-right" className="ml-1" />
        </Button>
      )}
    </div>
  );
}
